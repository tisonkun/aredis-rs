// Copyright 2022 tison <wander4096@gmail.com>.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use anyhow::Result;
use aredis::Client;

use crate::Utf8String;

struct Article {
    client: Client,
    id: String,
    title_key: String,
    content_key: String,
    author_key: String,
    created_at_key: String,
}

impl Article {
    pub fn new(client: Client, article_id: u64) -> Self {
        let id = article_id.to_string();
        let title_key = format!("article::{}::title", &id);
        let content_key = format!("article::{}::content", &id);
        let author_key = format!("article::{}::author", &id);
        let created_at_key = format!("article::{}::created_at", &id);
        Article {
            client,
            id,
            title_key,
            content_key,
            author_key,
            created_at_key,
        }
    }

    pub async fn create<T>(&mut self, title: T, content: T, author: T) -> Result<bool>
    where
        T: Into<String>,
    {
        let offset = time::macros::offset!(+8);
        let created_at = time::OffsetDateTime::now_utc().to_offset(offset);
        let article_data = vec![
            (self.title_key.as_str(), title.into()),
            (self.content_key.as_str(), content.into()),
            (self.author_key.as_str(), author.into()),
            (self.created_at_key.as_str(), created_at.to_string()),
        ];
        let result = self.client.msetnx(article_data).await?;
        Ok(result)
    }

    pub async fn get(&mut self) -> Result<HashMap<String, Option<String>>> {
        let keys = vec![
            self.title_key.as_str(),
            self.content_key.as_str(),
            self.author_key.as_str(),
            self.created_at_key.as_str(),
        ];
        let result = self.client.mget(keys).await?;
        assert_eq!(result.len(), 4);
        let article = HashMap::from([
            ("id".to_string(), Some(self.id.clone())),
            (
                "title".to_string(),
                result[0].clone().and_then(|e| String::from_utf8(e).ok()),
            ),
            (
                "content".to_string(),
                result[1].clone().and_then(|e| String::from_utf8(e).ok()),
            ),
            (
                "author".to_string(),
                result[2].clone().and_then(|e| String::from_utf8(e).ok()),
            ),
            (
                "created_at".to_string(),
                result[3].clone().and_then(|e| String::from_utf8(e).ok()),
            ),
        ]);
        Ok(article)
    }

    pub async fn update<T>(
        &mut self,
        title: Option<T>,
        content: Option<T>,
        author: Option<T>,
    ) -> Result<()>
    where
        T: Into<Vec<u8>>,
    {
        let mut article_data = vec![];
        if let Some(title) = title {
            article_data.push((self.title_key.as_str(), title));
        }
        if let Some(content) = content {
            article_data.push((self.content_key.as_str(), content));
        }
        if let Some(author) = author {
            article_data.push((self.author_key.as_str(), author));
        }
        assert!(!article_data.is_empty());
        self.client.mset(article_data).await?;
        Ok(())
    }

    pub async fn get_content_len(&mut self) -> Result<u64> {
        let result = self.client.strlen(self.content_key.as_str()).await?;
        Ok(result)
    }

    pub async fn get_content_preview<Out>(&mut self, preview_len: i64) -> Result<Out>
    where
        Out: From<Vec<u8>>,
    {
        let key = self.content_key.as_str();
        let start = 0;
        let end = preview_len - 1;
        let result = self.client.get_range(key, start, end).await?;
        Ok(result)
    }
}

#[tokio::test]
async fn test_article() -> Result<()> {
    let client = crate::client().await?;
    let mut article = Article::new(client, 42);

    let got = article.create("message", "hello world", "tisonkun").await?;
    assert!(got);

    let got = article.get().await?;
    assert_eq!(got.len(), 5);
    assert_eq!(got.get("author").unwrap(), &Some("tisonkun".to_string()));

    println!("article: {:?}", got);

    article.update(None, None, Some("brittani")).await?;

    let got = article.get().await?;
    assert_eq!(got.len(), 5);
    assert_eq!(got.get("author").unwrap(), &Some("brittani".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_article_preview() -> Result<()> {
    let client = crate::client().await?;
    let mut article = Article::new(client, 12345);

    let title = "Improving map data on GitHub";
    let content = "You've been able to view and diff geospatial data on GitHub for a while, \
        but now, in addition to being able to collaborate on the GeoJSON files \
        you upload to GitHub, you can now more easily contribute to the underlying, \
        shared basemap, that provides your data with context.";
    let author = "benbalter";

    let got = article.create(title, content, author).await?;
    assert!(got);

    let got = article.get_content_len().await?;
    assert_eq!(got, 273);

    let got: Utf8String = article.get_content_preview(100).await?;
    assert_eq!(got.0, content[0..100].to_string());
    Ok(())
}
