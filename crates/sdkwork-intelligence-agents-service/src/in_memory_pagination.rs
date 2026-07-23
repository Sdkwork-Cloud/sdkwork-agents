use crate::ports::{PaginationParams, SessionItemListSort};
use sdkwork_utils_rust::http_api::offset_limit_page_from_iter;

/// Bounded offset window over an ordered iterator (PAGINATION_SPEC §5.3 / §9).
pub(crate) fn paginate_iterator<I, T>(iter: I, pagination: &PaginationParams) -> Vec<T>
where
    I: Iterator<Item = T>,
{
    offset_limit_page_from_iter(iter, pagination.page_size, pagination.offset).items
}

/// Recent chat context: last N items in sequence order without materializing the session.
pub(crate) fn paginate_recent_context<I, T>(iter: I, page_size: usize) -> Vec<T>
where
    I: DoubleEndedIterator<Item = T>,
{
    let mut recent: Vec<T> = iter.rev().take(page_size).collect();
    recent.reverse();
    recent
}

pub(crate) fn count_iterator<I>(iter: I) -> u64
where
    I: Iterator,
{
    iter.count() as u64
}

pub(crate) fn paginate_items<I, T>(
    iter: I,
    pagination: &PaginationParams,
    sort: SessionItemListSort,
) -> Vec<T>
where
    I: DoubleEndedIterator<Item = T>,
{
    match sort {
        SessionItemListSort::SequenceAsc => paginate_iterator(iter, pagination),
        SessionItemListSort::SequenceDesc => paginate_iterator(iter.rev(), pagination),
        SessionItemListSort::RecentContextDesc => {
            paginate_recent_context(iter, pagination.page_size)
        }
    }
}
