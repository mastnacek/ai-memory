//! Reciprocal Rank Fusion: merging ranked result lists.

use super::SearchHit;
use crate::domain::FactId;
use std::collections::HashMap;

/// Merges keyword and semantic search hits using Reciprocal Rank Fusion (RRF).
pub fn reciprocal_rank_fusion(
    keyword_hits: Vec<SearchHit>,
    semantic_hits: Vec<SearchHit>,
    limit: usize,
) -> Vec<SearchHit> {
    if semantic_hits.is_empty() {
        let mut res = keyword_hits;
        res.truncate(limit);
        return res;
    }

    let mut rrf_map: HashMap<FactId, (f64, SearchHit)> = HashMap::new();
    let k = 60.0;

    for (rank, hit) in keyword_hits.into_iter().enumerate() {
        let score = 1.0 / (k + (rank as f64) + 1.0);
        rrf_map.insert(hit.id, (score, hit));
    }

    for (rank, hit) in semantic_hits.into_iter().enumerate() {
        let score = 1.0 / (k + (rank as f64) + 1.0);
        if let Some(entry) = rrf_map.get_mut(&hit.id) {
            entry.0 += score;
        } else {
            rrf_map.insert(hit.id, (score, hit));
        }
    }

    let mut final_hits: Vec<SearchHit> = rrf_map
        .into_values()
        .map(|(score, mut hit)| {
            hit.score = score;
            hit
        })
        .collect();

    final_hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    final_hits.truncate(limit);
    final_hits
}
