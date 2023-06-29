#![no_std]
#![no_main]

use core::slice::Iter;

use nanos_sdk::bindings::os_lib_end;
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);
use nanos_sdk::plugin::{
    PluginFeedParams, PluginFinalizeParams, PluginGetUiParams, PluginInitParams,
    PluginInteractionType, PluginQueryUiParams, PluginResult,
};
use nanos_sdk::string::String;
use nanos_sdk::{string, testing};
use starknet_sdk::types::{AbstractCall, AbstractCallData, FieldElement};

struct Selector {
    name: &'static str,
    value: [u8; 32],
}

struct StarknetIDCtx {
    domain: string::String<64>,
}

mod token;

#[no_mangle]
extern "C" fn sample_main(arg0: u32) {
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
            let call: &AbstractCall = unsafe { &*(params.data_in as *const AbstractCall) };

            if call.selector.value
                != [
                    0x02, 0xe2, 0x69, 0xd9, 0x30, 0xf6, 0xd7, 0xab, 0x92, 0xb1, 0x5c, 0xe8, 0xff,
                    0x9f, 0x5e, 0x63, 0x70, 0x93, 0x91, 0x61, 0x7e, 0x34, 0x65, 0xff, 0xf7, 0x9b,
                    0xa6, 0xba, 0xf2, 0x78, 0xce, 0x60,
                ]
            {
                // if the function called is not domain_to_address
                params.result = PluginResult::Err;
            } else {
                params.result = PluginResult::Ok;
            }
        }
        PluginInteractionType::Feed => {
            testing::debug_print("Feed plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFeedParams };

            let params: &mut PluginFeedParams = unsafe { &mut *value2 };
            let core_params = params.core_params.as_mut().unwrap();

            let starknetid_ctx = get_context(core_params.plugin_internal_ctx);

            let data_in = unsafe {
                &*(params.data_in as *const (&[AbstractCallData; 8], &[string::String<32>; 16]))
            };
            let calldata = data_in.0;
            let call_to_string = data_in.1;

            let domain_length = match calldata[0] {
                AbstractCallData::Felt(v) => v,
                _ => {
                    params.result = PluginResult::Err;
                    return;
                }
            };

            let calldata_slice = &calldata[1..(usize::from(domain_length))];

            match domain_as_str(calldata_slice.iter()) {
                Ok(domain_string) => {
                    starknetid_ctx.domain = domain_string;
                    params.result = PluginResult::Ok;
                }
                Err(_) => {
                    params.result = PluginResult::Err;
                }
            }
        }
        PluginInteractionType::Finalize => {
            testing::debug_print("Finalize plugin\n");
            let value2 = unsafe { *args.add(1) as *mut PluginFinalizeParams };
            let params: &mut PluginFinalizeParams = unsafe { &mut *value2 };
            params.result = PluginResult::Ok;
        }
        PluginInteractionType::QueryUi => {
            testing::debug_print("QueryUI plugin\n");
            let value2 = unsafe { *args.add(1) as *mut PluginQueryUiParams };
            let params: &mut PluginQueryUiParams = unsafe { &mut *value2 };
            // let _core_params = params.core_params.as_mut().unwrap();

            // let title = "ERC-20 OPERATION".as_bytes();
            // params.title.arr[..title.len()].copy_from_slice(title);
            // params.title.len = title.len();

            params.result = PluginResult::Ok;
        }
        PluginInteractionType::GetUi => {
            testing::debug_print("GetUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginGetUiParams };

            let params: &mut PluginGetUiParams = unsafe { &mut *value2 };
            // let core_params = params.core_params.as_mut().unwrap();

            // let erc20_ctx = get_context(core_params.plugin_internal_ctx);

            // testing::debug_print("requested screen index: ");
            // let s: string::String<2> = (params.ui_screen_idx as u8).into();
            // testing::debug_print(s.as_str());
            // testing::debug_print("\n");

            // let idx = erc20_ctx.token_info_idx.expect("unknown token");
            // let token = tokens[idx];

            // match params.ui_screen_idx {
            //     0 => {
            //         let title = "TOKEN:".as_bytes();
            //         params.title.arr[..title.len()].copy_from_slice(title);
            //         params.title.len = title.len();

            //         let msg = token.name.as_bytes();
            //         params.msg.arr[..msg.len()].copy_from_slice(msg);
            //         params.msg.len = msg.len();

            //         params.result = PluginResult::Ok;
            //     }
            //     1 => {
            //         let title = "METHOD:".as_bytes();
            //         params.title.arr[..title.len()].copy_from_slice(title);
            //         params.title.len = title.len();

            //         let msg = erc20_ctx.method.as_bytes();
            //         params.msg.arr[..msg.len()].copy_from_slice(msg);
            //         params.msg.len = msg.len();

            //         params.result = PluginResult::Ok;
            //     }
            //     2 => {
            //         let title = "TO:".as_bytes();
            //         params.title.arr[..title.len()].copy_from_slice(title);
            //         params.title.len = title.len();
            //         params.msg.arr[..erc20_ctx.destination.len]
            //             .copy_from_slice(&erc20_ctx.destination.arr[..erc20_ctx.destination.len]);
            //         params.msg.len = erc20_ctx.destination.len;

            //         params.result = PluginResult::Ok;
            //     }
            //     3 => {
            //         let title = "AMOUNT:".as_bytes();
            //         params.title.arr[..title.len()].copy_from_slice(title);
            //         params.title.len = title.len();

            //         let s = string::uint256_to_float(&erc20_ctx.amount, token.decimals);
            //         params.msg.arr[..s.len].copy_from_slice(&s.arr[..s.len]);
            //         params.msg.len = s.len;

            //         params.result = PluginResult::Ok;
            //     }
            //     _ => {
            //         params.result = PluginResult::Err;
            //     }
            // }
            params.result = PluginResult::Ok;
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

fn get_context(buf: *mut u8) -> &'static mut StarknetIDCtx {
    let addr = buf as usize;
    let alignment = core::mem::align_of::<StarknetIDCtx>();
    let offset: isize = (alignment - (addr % alignment)) as isize;
    let erc20_ctx: &mut StarknetIDCtx = unsafe { &mut *(buf.offset(offset) as *mut StarknetIDCtx) };

    erc20_ctx
}
