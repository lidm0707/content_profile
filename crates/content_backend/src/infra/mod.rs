pub mod http;
#[cfg(not(target_arch = "wasm32"))]
pub mod pg;
pub mod supabase;

use crate::app::{ContentDto, ContentPage, ContentTagService, TagDto};
use crate::app::{ContentService, TagService};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::{UpsertContent, UpsertTag};
use crate::port::{ContentRepo, TagRepo};
#[cfg(not(target_arch = "wasm32"))]
use supabase::sync;
use supabase::{
    SupabaseConfig, content::SupabaseContentRepo, content_tag::SupabaseContentTagRepo,
    tag::SupabaseTagRepo,
};

#[cfg(not(target_arch = "wasm32"))]
const MODE_PG: &str = "pg";
#[cfg(not(target_arch = "wasm32"))]
const MODE_PG_SYNC: &str = "pg_sync";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    #[cfg(not(target_arch = "wasm32"))]
    Pg,
    #[cfg(not(target_arch = "wasm32"))]
    PgSync,
    Supabase,
}

impl BackendMode {
    pub fn parse(mode: &str) -> Self {
        #[cfg(target_arch = "wasm32")]
        let _ = mode;
        #[cfg(not(target_arch = "wasm32"))]
        if mode == MODE_PG {
            return Self::Pg;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if mode == MODE_PG_SYNC {
            return Self::PgSync;
        }
        Self::Supabase
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type ContentBackendRepo = RepoSwitch<LocalContentRepo, SupabaseContentRepo>;
#[cfg(not(target_arch = "wasm32"))]
pub type TagBackendRepo = RepoSwitch<LocalTagRepo, SupabaseTagRepo>;
#[cfg(not(target_arch = "wasm32"))]
pub type ContentQueryRepo = RepoSwitch<LocalContentRepo, SupabaseContentRepo>;
#[cfg(not(target_arch = "wasm32"))]
pub type TagQueryRepo = RepoSwitch<LocalTagRepo, SupabaseTagRepo>;
#[cfg(target_arch = "wasm32")]
pub type ContentBackendRepo = SupabaseContentRepo;
#[cfg(target_arch = "wasm32")]
pub type TagBackendRepo = SupabaseTagRepo;
#[cfg(target_arch = "wasm32")]
pub type ContentQueryRepo = SupabaseContentRepo;
#[cfg(target_arch = "wasm32")]
pub type TagQueryRepo = SupabaseTagRepo;

#[derive(Clone)]
pub struct Backend {
    pub content: ContentService<ContentBackendRepo>,
    pub tag: TagService<TagBackendRepo>,
    pub content_tags: ContentTagService<SupabaseContentTagRepo>,
    content_query: ContentQueryRepo,
    tag_query: TagQueryRepo,
    config: Option<SupabaseConfig>,
    #[cfg(not(target_arch = "wasm32"))]
    content_sync: Option<sync::content::PgSyncContentRepo>,
    #[cfg(not(target_arch = "wasm32"))]
    tag_sync: Option<sync::tag::PgSyncTagRepo>,
}

impl PartialEq for Backend {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl Backend {
    pub fn new(mode: BackendMode, supabase: Option<SupabaseConfig>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let _ = mode;
        #[cfg(not(target_arch = "wasm32"))]
        let (content_repo, tag_repo, content_query, tag_query, content_sync, tag_sync) = match mode
        {
            BackendMode::Pg => {
                let content = LocalContentRepo::Pg(pg::content::PgContentRepo::from_env());
                let tag = LocalTagRepo::Pg(pg::tag::PgTagRepo::from_env());
                (
                    RepoSwitch::Pg(content.clone()),
                    RepoSwitch::Pg(tag.clone()),
                    RepoSwitch::Pg(content),
                    RepoSwitch::Pg(tag),
                    None,
                    None,
                )
            }
            BackendMode::PgSync => {
                let content = sync::content::PgSyncContentRepo::new(supabase.clone());
                let tag = sync::tag::PgSyncTagRepo::new(supabase.clone());
                let local_content = LocalContentRepo::PgSync(content.clone());
                let local_tag = LocalTagRepo::PgSync(tag.clone());
                (
                    RepoSwitch::Pg(local_content.clone()),
                    RepoSwitch::Pg(local_tag.clone()),
                    RepoSwitch::Pg(local_content),
                    RepoSwitch::Pg(local_tag),
                    Some(content),
                    Some(tag),
                )
            }
            BackendMode::Supabase => {
                let content = SupabaseContentRepo::new(supabase.clone());
                let tag = SupabaseTagRepo::new(supabase.clone());
                (
                    RepoSwitch::Supabase(content.clone()),
                    RepoSwitch::Supabase(tag.clone()),
                    RepoSwitch::Supabase(content),
                    RepoSwitch::Supabase(tag),
                    None,
                    None,
                )
            }
        };
        #[cfg(target_arch = "wasm32")]
        let content_repo = SupabaseContentRepo::new(supabase.clone());
        #[cfg(target_arch = "wasm32")]
        let tag_repo = SupabaseTagRepo::new(supabase.clone());
        #[cfg(target_arch = "wasm32")]
        let content_query = SupabaseContentRepo::new(supabase.clone());
        #[cfg(target_arch = "wasm32")]
        let tag_query = SupabaseTagRepo::new(supabase.clone());

        let content_tags = ContentTagService::new(SupabaseContentTagRepo::new(supabase.clone()));

        Self {
            content: ContentService::new(content_repo),
            tag: TagService::new(tag_repo),
            content_tags,
            content_query,
            tag_query,
            config: supabase,
            #[cfg(not(target_arch = "wasm32"))]
            content_sync,
            #[cfg(not(target_arch = "wasm32"))]
            tag_sync,
        }
    }

    pub async fn content_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        self.content_query.list_by_ids(ids).await
    }

    pub async fn content_page(&self, page: u32, page_size: u32) -> Result<ContentPage, String> {
        let page_size = page_size.max(1);
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let items = self.content_query.list_page(offset, page_size).await?;
        let total_items = self.content_query.count().await?;
        let total_pages = total_items.div_ceil(page_size);
        Ok(ContentPage {
            page,
            page_size,
            items,
            total_items,
            total_pages,
        })
    }

    pub async fn tags_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        self.tag_query.list_by_ids(ids).await
    }

    pub async fn tags_for_content(&self, content_id: i32) -> Result<Vec<TagDto>, String> {
        let links = self.content_tags.list_for_content(content_id).await?;
        if links.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<i32> = links.iter().map(|row| row.tag_id).collect();
        self.tag_query.list_by_ids(&ids).await
    }

    /// Push queued local writes to Supabase. No-op outside pg_sync mode.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn flush_pending(&self) -> Result<usize, String> {
        let mut pushed = 0;
        if let Some(content) = &self.content_sync {
            pushed += content.flush_pending().await?;
        }
        if let Some(tag) = &self.tag_sync {
            pushed += tag.flush_pending().await?;
        }
        Ok(pushed)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn flush_pending(&self) -> Result<usize, String> {
        Ok(0)
    }

    /// Pulls content and tags from Supabase into local pg. No-op outside
    /// pg_sync mode.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn pull_remote(&self) -> Result<usize, String> {
        let mut pulled = 0;
        if let Some(content) = &self.content_sync {
            pulled += content.pull_remote().await?;
        }
        if let Some(tag) = &self.tag_sync {
            pulled += tag.pull_remote().await?;
        }
        Ok(pulled)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn pull_remote(&self) -> Result<usize, String> {
        Ok(0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub enum LocalContentRepo {
    Pg(pg::content::PgContentRepo),
    PgSync(sync::content::PgSyncContentRepo),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub enum LocalTagRepo {
    Pg(pg::tag::PgTagRepo),
    PgSync(sync::tag::PgSyncTagRepo),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub enum RepoSwitch<L, S> {
    Pg(L),
    Supabase(S),
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl ContentRepo for LocalContentRepo {
    async fn list(&self) -> Result<Vec<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.list().await,
            Self::PgSync(repo) => repo.list().await,
        }
    }

    async fn get(&self, id: i32) -> Result<Option<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.get(id).await,
            Self::PgSync(repo) => repo.get(id).await,
        }
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.get_by_slug(slug).await,
            Self::PgSync(repo) => repo.get_by_slug(slug).await,
        }
    }

    async fn create(&self, data: UpsertContent) -> Result<ContentDto, String> {
        match self {
            Self::Pg(repo) => repo.create(data).await,
            Self::PgSync(repo) => repo.create(data).await,
        }
    }

    async fn update(&self, id: i32, data: UpsertContent) -> Result<ContentDto, String> {
        match self {
            Self::Pg(repo) => repo.update(id, data).await,
            Self::PgSync(repo) => repo.update(id, data).await,
        }
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        match self {
            Self::Pg(repo) => repo.delete(id).await,
            Self::PgSync(repo) => repo.delete(id).await,
        }
    }

    async fn list_page(&self, offset: u32, limit: u32) -> Result<Vec<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.list_page(offset, limit).await,
            Self::PgSync(repo) => repo.list_page(offset, limit).await,
        }
    }

    async fn count(&self) -> Result<u32, String> {
        match self {
            Self::Pg(repo) => repo.count().await,
            Self::PgSync(repo) => repo.count().await,
        }
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.list_by_ids(ids).await,
            Self::PgSync(repo) => repo.list_by_ids(ids).await,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl TagRepo for LocalTagRepo {
    async fn list(&self) -> Result<Vec<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.list().await,
            Self::PgSync(repo) => repo.list().await,
        }
    }

    async fn get(&self, id: i32) -> Result<Option<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.get(id).await,
            Self::PgSync(repo) => repo.get(id).await,
        }
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.get_by_slug(slug).await,
            Self::PgSync(repo) => repo.get_by_slug(slug).await,
        }
    }

    async fn create(&self, data: UpsertTag) -> Result<TagDto, String> {
        match self {
            Self::Pg(repo) => repo.create(data).await,
            Self::PgSync(repo) => repo.create(data).await,
        }
    }

    async fn update(&self, id: i32, data: UpsertTag) -> Result<TagDto, String> {
        match self {
            Self::Pg(repo) => repo.update(id, data).await,
            Self::PgSync(repo) => repo.update(id, data).await,
        }
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        match self {
            Self::Pg(repo) => repo.delete(id).await,
            Self::PgSync(repo) => repo.delete(id).await,
        }
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.list_by_ids(ids).await,
            Self::PgSync(repo) => repo.list_by_ids(ids).await,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl<L: ContentRepo, S: ContentRepo> ContentRepo for RepoSwitch<L, S> {
    async fn list(&self) -> Result<Vec<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.list().await,
            Self::Supabase(repo) => repo.list().await,
        }
    }

    async fn get(&self, id: i32) -> Result<Option<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.get(id).await,
            Self::Supabase(repo) => repo.get(id).await,
        }
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.get_by_slug(slug).await,
            Self::Supabase(repo) => repo.get_by_slug(slug).await,
        }
    }

    async fn create(&self, data: UpsertContent) -> Result<ContentDto, String> {
        match self {
            Self::Pg(repo) => repo.create(data).await,
            Self::Supabase(repo) => repo.create(data).await,
        }
    }

    async fn update(&self, id: i32, data: UpsertContent) -> Result<ContentDto, String> {
        match self {
            Self::Pg(repo) => repo.update(id, data).await,
            Self::Supabase(repo) => repo.update(id, data).await,
        }
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        match self {
            Self::Pg(repo) => repo.delete(id).await,
            Self::Supabase(repo) => repo.delete(id).await,
        }
    }

    async fn list_page(&self, offset: u32, limit: u32) -> Result<Vec<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.list_page(offset, limit).await,
            Self::Supabase(repo) => repo.list_page(offset, limit).await,
        }
    }

    async fn count(&self) -> Result<u32, String> {
        match self {
            Self::Pg(repo) => repo.count().await,
            Self::Supabase(repo) => repo.count().await,
        }
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        match self {
            Self::Pg(repo) => repo.list_by_ids(ids).await,
            Self::Supabase(repo) => repo.list_by_ids(ids).await,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl<L: TagRepo, S: TagRepo> TagRepo for RepoSwitch<L, S> {
    async fn list(&self) -> Result<Vec<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.list().await,
            Self::Supabase(repo) => repo.list().await,
        }
    }

    async fn get(&self, id: i32) -> Result<Option<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.get(id).await,
            Self::Supabase(repo) => repo.get(id).await,
        }
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.get_by_slug(slug).await,
            Self::Supabase(repo) => repo.get_by_slug(slug).await,
        }
    }

    async fn create(&self, data: UpsertTag) -> Result<TagDto, String> {
        match self {
            Self::Pg(repo) => repo.create(data).await,
            Self::Supabase(repo) => repo.create(data).await,
        }
    }

    async fn update(&self, id: i32, data: UpsertTag) -> Result<TagDto, String> {
        match self {
            Self::Pg(repo) => repo.update(id, data).await,
            Self::Supabase(repo) => repo.update(id, data).await,
        }
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        match self {
            Self::Pg(repo) => repo.delete(id).await,
            Self::Supabase(repo) => repo.delete(id).await,
        }
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        match self {
            Self::Pg(repo) => repo.list_by_ids(ids).await,
            Self::Supabase(repo) => repo.list_by_ids(ids).await,
        }
    }
}
