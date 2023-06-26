#![no_std]
#![no_main]

use core::slice::Iter;

use nanos_sdk::bindings::os_lib_end;

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

use nanos_sdk::plugin::{
    PluginCheckParams, PluginCoreParams, PluginFeedParams, PluginFinalizeParams, PluginGetUiParams,
    PluginInitParams, PluginInteractionType, PluginQueryUiParams, PluginResult,
};

use nanos_sdk::string::String;
use nanos_sdk::{string, testing};

use starknet_sdk::types::{AbstractCall, AbstractCallData, Call, FieldElement, TransactionInfo};

struct Selector {
    name: &'static str,
    value: [u8; 32],
}

struct Erc20Ctx {
    address: [u8; 32],
    method: &'static str,
    destination: string::String<64>,
    amount: [u8; 32],
    token_info_idx: Option<usize>,
}

const N_SELECTORS: usize = 2;

const METHODS: [&str; N_SELECTORS] = ["TRANSFER", "APPROVE"];
const SN_KECCAK: [[u8; 32]; N_SELECTORS] = [
    [
        0x00, 0x83, 0xaf, 0xd3, 0xf4, 0xca, 0xed, 0xc6, 0xee, 0xbf, 0x44, 0x24, 0x6f, 0xe5, 0x4e,
        0x38, 0xc9, 0x5e, 0x31, 0x79, 0xa5, 0xec, 0x9e, 0xa8, 0x17, 0x40, 0xec, 0xa5, 0xb4, 0x82,
        0xd1, 0x2e,
    ],
    [
        0x02, 0x19, 0x20, 0x9e, 0x08, 0x32, 0x75, 0x17, 0x17, 0x74, 0xda, 0xb1, 0xdf, 0x80, 0x98,
        0x2e, 0x9d, 0xf2, 0x09, 0x65, 0x16, 0xf0, 0x63, 0x19, 0xc5, 0xc6, 0xd7, 0x1a, 0xe0, 0xa8,
        0x48, 0x0c,
    ],
];

const SELECTORS: [Selector; N_SELECTORS] = [
    Selector {
        name: "transfer",
        value: [
            0x00, 0x83, 0xaf, 0xd3, 0xf4, 0xca, 0xed, 0xc6, 0xee, 0xbf, 0x44, 0x24, 0x6f, 0xe5,
            0x4e, 0x38, 0xc9, 0x5e, 0x31, 0x79, 0xa5, 0xec, 0x9e, 0xa8, 0x17, 0x40, 0xec, 0xa5,
            0xb4, 0x82, 0xd1, 0x2e,
        ],
    },
    Selector {
        name: "approve",
        value: [
            0x02, 0x19, 0x20, 0x9e, 0x08, 0x32, 0x75, 0x17, 0x17, 0x74, 0xda, 0xb1, 0xdf, 0x80,
            0x98, 0x2e, 0x9d, 0xf2, 0x09, 0x65, 0x16, 0xf0, 0x63, 0x19, 0xc5, 0xc6, 0xd7, 0x1a,
            0xe0, 0xa8, 0x48, 0x0c,
        ],
    },
];

mod token;
use token::{TokenInfo, TOKENS};

#[no_mangle]
extern "C" fn sample_main(arg0: u32) {
    // to remove when PR https://github.com/LedgerHQ/ledger-nanos-sdk/pull/69 will be merged into SDK
    let selectors: [Selector; 2] = [
        Selector {
            name: METHODS[0],
            value: SN_KECCAK[0],
        },
        Selector {
            name: METHODS[1],
            value: SN_KECCAK[1],
        },
    ];

    // to remove when PR https://github.com/LedgerHQ/ledger-nanos-sdk/pull/69 will be merged into SDK
    let tokens: [TokenInfo; 2] = [
        TokenInfo {
            address: [
                0x06, 0x8f, 0x5c, 0x6a, 0x61, 0x78, 0x07, 0x68, 0x45, 0x5d, 0xe6, 0x90, 0x77, 0xe0,
                0x7e, 0x89, 0x78, 0x78, 0x39, 0xbf, 0x81, 0x66, 0xde, 0xcf, 0xbf, 0x92, 0xb6, 0x45,
                0x20, 0x9c, 0x0f, 0xb8,
            ],
            name: "Tether USDT",
            ticker: "USDT".as_bytes(),
            decimals: 6,
        },
        TokenInfo {
            address: [
                0x04, 0x9d, 0x36, 0x57, 0x0d, 0x4e, 0x46, 0xf4, 0x8e, 0x99, 0x67, 0x4b, 0xd3, 0xfc,
                0xc8, 0x46, 0x44, 0xdd, 0xd6, 0xb9, 0x6f, 0x7c, 0x74, 0x1b, 0x15, 0x62, 0xb8, 0x2f,
                0x9e, 0x00, 0x4d, 0xc7,
            ],
            name: "Ether",
            ticker: "ETH".as_bytes(),
            decimals: 18,
        },
    ];

    let args: *mut u32 = arg0 as *mut u32;

    let value1 = unsafe { *args as u16 };
    let operation: PluginInteractionType = value1.into();

    match operation {
        PluginInteractionType::Check => {
            testing::debug_print("Check plugin presence\n");
        }
        PluginInteractionType::Init => {
            testing::debug_print("Init plugin context\n");

            let value2 = unsafe { *args.add(1) as *mut PluginInitParams };

            let params: &mut PluginInitParams = unsafe { &mut *value2 };
            let core_params = params.core_params.as_mut().unwrap();

            let erc20_ctx: &mut Erc20Ctx =
                unsafe { &mut *(core_params.plugin_internal_ctx as *mut Erc20Ctx) };
            let call: &AbstractCall = unsafe { &*(params.data_in as *const AbstractCall) };

            erc20_ctx.address = call.to.value;
            for i in 0..N_SELECTORS {
                if call.selector.value == selectors[i].value {
                    erc20_ctx.method = selectors[i].name;
                }
            }
            params.result = PluginResult::Ok;
        }
        PluginInteractionType::Feed => {
            testing::debug_print("Feed plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFeedParams };

            let params: &mut PluginFeedParams = unsafe { &mut *value2 };
            let core_params = params.core_params.as_mut().unwrap();

            let erc20_ctx: &mut Erc20Ctx =
                unsafe { &mut *(core_params.plugin_internal_ctx as *mut Erc20Ctx) };

            let data_in = unsafe {
                &*(params.data_in as *const (&[AbstractCallData; 8], &[string::String<32>; 16]))
            };
            let calldata = data_in.0;
            let call_to_string = data_in.1;

            match calldata[0] {
                AbstractCallData::Felt(v) => {
                    erc20_ctx.destination = v.value.into();
                }
                AbstractCallData::CallRef(idx, shift) => {
                    let s = call_to_string[idx];
                    for i in 0..s.len {
                        erc20_ctx.destination.arr[i] = s.arr[i];
                    }
                    erc20_ctx.destination.len = s.len;
                }
                _ => (),
            };

            erc20_ctx.amount = match calldata[1] {
                AbstractCallData::Felt(v) => v.value,
                _ => FieldElement::ZERO.value,
            };

            {
                testing::debug_print("Token: 0x");
                let s: string::String<64> = erc20_ctx.address.into();
                testing::debug_print(s.as_str());
                testing::debug_print("\n");

                testing::debug_print("method: ");
                testing::debug_print(erc20_ctx.method);
                testing::debug_print("\n");

                testing::debug_print("destination: ");
                testing::debug_print(erc20_ctx.destination.as_str());
                testing::debug_print("\n");

                testing::debug_print("amount: ");
                let s = string::uint256_to_float(&erc20_ctx.amount, 18);
                testing::debug_print(s.as_str());
                testing::debug_print("\n");
            }
            params.result = PluginResult::Ok;
        }
        PluginInteractionType::Finalize => {
            testing::debug_print("Finalize plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFinalizeParams };

            let params: &mut PluginFinalizeParams = unsafe { &mut *value2 };
            let core_params = params.core_params.as_mut().unwrap();

            let erc20_ctx: &mut Erc20Ctx =
                unsafe { &mut *(core_params.plugin_internal_ctx as *mut Erc20Ctx) };

            erc20_ctx.token_info_idx = None;
            for i in 0..2 {
                if erc20_ctx.address == tokens[i].address {
                    erc20_ctx.token_info_idx = Some(i);
                }
            }
            params.num_ui_screens = 4;
            params.result = match erc20_ctx.token_info_idx {
                Some(_idx) => {
                    testing::debug_print("token info found in plugin\n");
                    PluginResult::Ok
                }
                None => {
                    testing::debug_print("token info not found in plugin\n");
                    PluginResult::NeedInfo
                }
            };
        }
        PluginInteractionType::QueryUi => {
            testing::debug_print("QueryUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginQueryUiParams };

            let params: &mut PluginQueryUiParams = unsafe { &mut *value2 };
            let _core_params = params.core_params.as_mut().unwrap();

            let title = "ERC-20 OPERATION".as_bytes();
            params.title.arr[..title.len()].copy_from_slice(title);
            params.title.len = title.len();

            params.result = PluginResult::Ok;
        }
        PluginInteractionType::GetUi => {
            testing::debug_print("GetUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginGetUiParams };

            let params: &mut PluginGetUiParams = unsafe { &mut *value2 };
            let core_params = params.core_params.as_mut().unwrap();

            let erc20_ctx: &mut Erc20Ctx =
                unsafe { &mut *(core_params.plugin_internal_ctx as *mut Erc20Ctx) };

            testing::debug_print("requested screen index: ");
            let s: string::String<2> = (params.ui_screen_idx as u8).into();
            testing::debug_print(s.as_str());
            testing::debug_print("\n");

            let idx = erc20_ctx.token_info_idx.expect("unknown token");
            let token = tokens[idx];

            match params.ui_screen_idx {
                0 => {
                    let title = "TOKEN:".as_bytes();
                    params.title.arr[..title.len()].copy_from_slice(title);
                    params.title.len = title.len();

                    let msg = token.name.as_bytes();
                    params.msg.arr[..msg.len()].copy_from_slice(msg);
                    params.msg.len = msg.len();

                    params.result = PluginResult::Ok;
                }
                1 => {
                    let title = "METHOD:".as_bytes();
                    params.title.arr[..title.len()].copy_from_slice(title);
                    params.title.len = title.len();

                    let msg = erc20_ctx.method.as_bytes();
                    params.msg.arr[..msg.len()].copy_from_slice(msg);
                    params.msg.len = msg.len();

                    params.result = PluginResult::Ok;
                }
                2 => {
                    let title = "TO:".as_bytes();
                    params.title.arr[..title.len()].copy_from_slice(title);
                    params.title.len = title.len();
                    params.msg.arr[..erc20_ctx.destination.len]
                        .copy_from_slice(&erc20_ctx.destination.arr[..erc20_ctx.destination.len]);
                    params.msg.len = erc20_ctx.destination.len;

                    params.result = PluginResult::Ok;
                }
                3 => {
                    let title = "AMOUNT:".as_bytes();
                    params.title.arr[..title.len()].copy_from_slice(title);
                    params.title.len = title.len();

                    let s = string::uint256_to_float(&erc20_ctx.amount, token.decimals);
                    params.msg.arr[..s.len].copy_from_slice(&s.arr[..s.len]);
                    params.msg.len = s.len;

                    params.result = PluginResult::Ok;
                }
                _ => {
                    params.result = PluginResult::Err;
                }
            }
        }
        _ => {
            testing::debug_print("Not implemented\n");
        }
    }
    unsafe {
        os_lib_end();
    }
}

// default alphabet + escape = 38
const DEFAULT_DIVIDER: FieldElement = FieldElement {
    value: [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x26,
    ],
};
// escape = 37
const ESCAPE: FieldElement = FieldElement {
    value: [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x25,
    ],
};

const LETTERS_LEN: FieldElement = FieldElement {
    value: [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x1a,
    ],
};

#[derive(Debug)]
enum DecodeError {
    UnsupportedAlphabet,
    OutOfCapacity,
}

fn domain_as_str(calldatas: Iter<AbstractCallData>) -> Result<String<64>, DecodeError> {
    let mut output: String<64> = String::new();
    for calldata in calldatas {
        if output.len == output.capacity {
            return Err(DecodeError::OutOfCapacity);
        }
        match calldata {
            AbstractCallData::Felt(felt) => append_decoded(felt.clone(), &mut output)?,
            _ => {}
        }
        output.arr[output.len] = b'.';
        output.len += 1;
    }
    if output.len + 5 > output.capacity {
        return Err(DecodeError::OutOfCapacity);
    }
    output.arr[output.len] = b's';
    output.arr[output.len + 1] = b't';
    output.arr[output.len + 2] = b'a';
    output.arr[output.len + 3] = b'r';
    output.arr[output.len + 4] = b'k';
    output.len += 5;
    Ok(output)
}

fn append_decoded(mut felt: FieldElement, output: &mut String<64>) -> Result<(), DecodeError> {
    while felt != FieldElement::ZERO {
        if output.len == output.capacity {
            return Err(DecodeError::OutOfCapacity);
        }
        let (q, r) = (&felt).div_rem(&DEFAULT_DIVIDER);
        felt = q;
        if r == ESCAPE {
            return Err(DecodeError::UnsupportedAlphabet);
        }

        let byte: u8 = r.into();
        output.arr[output.len] = byte + if r < LETTERS_LEN { 97u8 } else { 22u8 };
        output.len += 1;
    }
    return Ok(());
}
