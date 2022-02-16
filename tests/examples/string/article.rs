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

    pub async fn create<T: Into<Vec<u8>>>(
        &mut self,
        title: T,
        content: T,
        author: T,
    ) -> Result<bool> {
        let offset = time::macros::offset!(+8);
        let created_at = time::OffsetDateTime::now_utc().to_offset(offset);
        let article_data = vec![
            (self.title_key.as_bytes().to_vec(), title.into()),
            (self.content_key.as_bytes().to_vec(), content.into()),
            (self.author_key.as_bytes().to_vec(), author.into()),
            (
                self.created_at_key.as_bytes().to_vec(),
                created_at.to_string().into_bytes(),
            ),
        ];
        let result = self.client.msetnx(article_data).await?;
        Ok(result)
    }

    pub async fn get(&mut self) -> Result<HashMap<String, Option<String>>> {
        let keys = vec![
            self.title_key.as_bytes().to_vec(),
            self.content_key.as_bytes().to_vec(),
            self.author_key.as_bytes().to_vec(),
            self.created_at_key.as_bytes().to_vec(),
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

    pub async fn update<T: Into<Vec<u8>>>(
        &mut self,
        title: Option<T>,
        content: Option<T>,
        author: Option<T>,
    ) -> Result<()> {
        let mut article_data = vec![];
        if let Some(title) = title {
            article_data.push((self.title_key.as_bytes().to_vec(), title.into()));
        }
        if let Some(content) = content {
            article_data.push((self.content_key.as_bytes().to_vec(), content.into()));
        }
        if let Some(author) = author {
            article_data.push((self.author_key.as_bytes().to_vec(), author.into()));
        }
        assert!(!article_data.is_empty());
        self.client.mset(article_data).await?;
        Ok(())
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
