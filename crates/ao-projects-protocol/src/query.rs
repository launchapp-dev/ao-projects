use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageRequest {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

impl<T> Page<T> {
    pub fn from_vec(items: Vec<T>, total: usize, offset: usize, limit: usize) -> Self {
        Self {
            has_more: offset + items.len() < total,
            items,
            total,
            offset,
            limit,
        }
    }
}

pub fn paginate<T>(items: Vec<T>, page: &PageRequest) -> Page<T> {
    let total = items.len();
    let start = page.offset.min(total);
    let end = (start + page.limit).min(total);
    let page_items: Vec<T> = items.into_iter().skip(start).take(end - start).collect();
    Page::from_vec(page_items, total, page.offset, page.limit)
}
