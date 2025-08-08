// Copyright 2016 Brendan Zabarauskas and the gl-rs developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod gl {
    #![allow(
        clippy::missing_safety_doc,
        clippy::missing_transmute_annotations,
        clippy::too_many_arguments,
        clippy::unused_unit
    )]
    include!(concat!(env!("OUT_DIR"), "/test_add_registries.rs"));
}
