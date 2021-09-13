/*
 * Copyright (c) 2021 Works Applications Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::Deserialize;
use serde_json::Value;

use sudachi::config::Config;
use sudachi::declare_oov_provider_plugin;
use sudachi::dic::grammar::Grammar;
use sudachi::dic::lexicon::word_infos::WordInfo;
use sudachi::input_text::Utf8InputText;
use sudachi::lattice::node::Node;
use sudachi::plugin::oov::OovProviderPlugin;
use sudachi::prelude::*;

declare_oov_provider_plugin!(SimpleOovPlugin, SimpleOovPlugin::default);

/// Provides a OOV node with single character if no words found in the dictionary
#[derive(Default)]
pub struct SimpleOovPlugin {
    left_id: i16,
    right_id: i16,
    cost: i16,
    oov_pos_id: u16,
}

/// Struct corresponds with raw config json file.
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PluginSettings {
    oovPOS: Vec<String>,
    leftId: i16,
    rightId: i16,
    cost: i16,
}

impl OovProviderPlugin for SimpleOovPlugin {
    fn set_up(
        &mut self,
        settings: &Value,
        _config: &Config,
        grammar: &Grammar,
    ) -> SudachiResult<()> {
        let settings: PluginSettings = serde_json::from_value(settings.clone())?;

        let oov_pos_string: Vec<&str> = settings.oovPOS.iter().map(|s| s.as_str()).collect();
        let oov_pos_id = grammar.get_part_of_speech_id(&oov_pos_string).ok_or(
            SudachiError::InvalidPartOfSpeech(format!("{:?}", oov_pos_string)),
        )?;
        let left_id = settings.leftId;
        let right_id = settings.rightId;
        let cost = settings.cost;

        self.oov_pos_id = oov_pos_id;
        self.left_id = left_id;
        self.right_id = right_id;
        self.cost = cost;

        Ok(())
    }

    fn provide_oov(
        &self,
        input_text: &Utf8InputText,
        offset: usize,
        has_other_words: bool,
    ) -> SudachiResult<Vec<Node>> {
        if has_other_words {
            return Ok(vec![]);
        }

        let length = input_text.get_word_candidate_length(offset);
        let surface = input_text.get_substring(offset, offset + length)?;

        Ok(vec![Node::new_oov(
            self.left_id,
            self.right_id,
            self.cost,
            WordInfo {
                normalized_form: surface.clone(),
                dictionary_form: surface.clone(),
                surface,
                head_word_length: length as u16,
                pos_id: self.oov_pos_id,
                dictionary_form_word_id: -1,
                ..Default::default()
            },
        )])
    }
}
