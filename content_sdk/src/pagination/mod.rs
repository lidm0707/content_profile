//! Pagination support for content_sdk

use serde::{Deserialize, Serialize};

pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_PAGE_SIZE: u32 = 10;
pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        let validated_page = page.max(1);
        let validated_page_size = page_size.clamp(1, MAX_PAGE_SIZE);

        Self {
            page: validated_page,
            page_size: validated_page_size,
        }
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_items: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_previous(&self) -> bool {
        self.page > 1
    }

    pub fn empty(params: &PaginationParams) -> Self {
        Self {
            data: Vec::new(),
            page: params.page,
            page_size: params.page_size,
            total_items: 0,
            total_pages: 0,
        }
    }

    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: u32) -> Self {
        let total_pages = if total_items == 0 {
            0
        } else {
            ((total_items - 1) / params.page_size) + 1
        };

        Self {
            data,
            page: params.page,
            page_size: params.page_size,
            total_items,
            total_pages,
        }
    }
}
