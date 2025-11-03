// Copyright 2015 Brendan Zabarauskas and the gl-rs developers
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
    include!(concat!(env!("OUT_DIR"), "/test_symbols.rs"));
}

pub fn compile_test_symbols_exist() {
    let _ = gl::DebugMessageControlARB;
    let _ = gl::DebugMessageInsertARB;
    let _ = gl::DebugMessageCallbackARB;
    let _ = gl::GetDebugMessageLogARB;

    assert_eq!(gl::DEBUG_OUTPUT_SYNCHRONOUS_ARB, 0x8242);
    assert_eq!(gl::MAX_DEBUG_MESSAGE_LENGTH_ARB, 0x9143);
    assert_eq!(gl::MAX_DEBUG_LOGGED_MESSAGES_ARB, 0x9144);
    assert_eq!(gl::DEBUG_LOGGED_MESSAGES_ARB, 0x9145);
    assert_eq!(gl::DEBUG_NEXT_LOGGED_MESSAGE_LENGTH_ARB, 0x8243);
    assert_eq!(gl::DEBUG_CALLBACK_FUNCTION_ARB, 0x8244);
    assert_eq!(gl::DEBUG_CALLBACK_USER_PARAM_ARB, 0x8245);
    assert_eq!(gl::DEBUG_SOURCE_API_ARB, 0x8246);
    assert_eq!(gl::DEBUG_SOURCE_WINDOW_SYSTEM_ARB, 0x8247);
    assert_eq!(gl::DEBUG_SOURCE_SHADER_COMPILER_ARB, 0x8248);
    assert_eq!(gl::DEBUG_SOURCE_THIRD_PARTY_ARB, 0x8249);
    assert_eq!(gl::DEBUG_SOURCE_APPLICATION_ARB, 0x824A);
    assert_eq!(gl::DEBUG_SOURCE_OTHER_ARB, 0x824B);
    assert_eq!(gl::DEBUG_TYPE_ERROR_ARB, 0x824C);
    assert_eq!(gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR_ARB, 0x824D);
    assert_eq!(gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR_ARB, 0x824E);
    assert_eq!(gl::DEBUG_TYPE_PORTABILITY_ARB, 0x824F);
    assert_eq!(gl::DEBUG_TYPE_PERFORMANCE_ARB, 0x8250);
    assert_eq!(gl::DEBUG_TYPE_OTHER_ARB, 0x8251);
    assert_eq!(gl::DEBUG_SEVERITY_HIGH_ARB, 0x9146);
    assert_eq!(gl::DEBUG_SEVERITY_MEDIUM_ARB, 0x9147);
    assert_eq!(gl::DEBUG_SEVERITY_LOW_ARB, 0x9148);
}
