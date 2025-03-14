// SPDX-FileCopyrightText: Copyright (c) 2024-2025 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
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

pub mod layer;
pub mod manager;
pub mod reserved;
pub mod reuse;
pub mod sequence;
pub mod storage;

// #[cfg(feature = "cuda_kv")]
// pub mod storage;

use reserved::*;

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    sync::{atomic::AtomicU64, Arc, RwLock},
};

use async_trait::async_trait;
use derive_getters::Dissolve;
use dynamo_runtime::{
    raise,
    utils::pool::{PoolExt, PoolItem, PoolValue, Returnable, SharedPoolItem},
    Result,
};

use crate::tokens::{PartialTokenBlock, SequenceHash, TokenBlock, Tokens};

use tracing as log;

pub type UniqueBlock<T> = PoolItem<KvBlock<T>>;
pub type SharedBlock<T> = SharedPoolItem<KvBlock<T>>;

#[derive(Debug, Clone, Default)]
pub struct NullStorage {}

impl BlockStorage for NullStorage {
    fn layer_pointer(&self, layer_id: usize) -> u64 {
        0
    }
    fn layer_size_in_bytes(&self) -> usize {
        0
    }
}
pub trait BlockStorage {
    fn layer_pointer(&self, layer_id: usize) -> u64 {
        0
    }
    fn layer_size_in_bytes(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone, Default)]
pub struct KvBlock<T: BlockStorage + Send + Sync> {
    token_block: TokenBlock,
    priority: u32,
    return_tick: u64,
    storage: T,
}

// pub struct KvStorage {
//     data: u64,
//     size: usize,

//     layer_idx: usize,
//     block_idx: usize,

//     /// The layout of the tensor
//     layout: layer::KvLayer,
// }

impl<T: BlockStorage + Send + Sync> KvBlock<T> {
    /// Creates a new KvBlock with the given token block
    pub fn new(token_block: TokenBlock) -> KvBlock<NullStorage> {
        let storage = NullStorage {};
        KvBlock {
            token_block,
            priority: 0,
            return_tick: 0,
            storage,
        }
    }

    /// Updates the token block
    pub fn update_token_block(&mut self, token_block: TokenBlock) {
        self.token_block = token_block;
    }

    /// Resets the block to its initial state
    pub(crate) fn reset(&mut self) {
        self.token_block = TokenBlock::default();
        self.priority = 0;
        self.return_tick = 0;
        // self.storage = None;
        // self.storage_state = StorageState::Absent;
    }
}

impl<T: BlockStorage + Send + Sync + 'static> Returnable for KvBlock<T> {
    fn on_return(&mut self) {}
}

pub struct KvBlockConfig {}
