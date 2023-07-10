#![no_std]
#![no_main]

use core::slice::Iter;

use nanos_sdk::bindings::os_lib_end;
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);
use nanos_sdk::plugin::{PluginInteractionType, PluginParam, PluginResult};
use nanos_sdk::string::String;
use nanos_sdk::{string, testing};
use starknet_sdk::types::{AbstractCall, AbstractCallData, FieldElement, UiParam};

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
            testing::debug_print("starknet-id: Init plugin context\n");

            let value2 = unsafe { *args.add(1) as *mut PluginParam };

            let params: &mut PluginParam = unsafe { &mut *value2 };
            let starknetid_ctx =
                get_context(params.plugin_internal_ctx, params.plugin_internal_ctx_len)
                    .expect("error when getting ctx");

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

            let value2 = unsafe { *args.add(1) as *mut PluginParam };

            let params: &mut PluginParam = unsafe { &mut *value2 };
            let starknetid_ctx =
                get_context(params.plugin_internal_ctx, params.plugin_internal_ctx_len)
                    .expect("error when getting ctx");

            let data_in = unsafe {
                &*(params.data_in as *const (&[AbstractCallData; 8], &[string::String<32>; 16]))
            };
            let calldata = data_in.0;
            let call_to_string = data_in.1;

            let domain_length = match calldata[0] {
                AbstractCallData::Felt(v) => v,
                _ => {
                    testing::debug_print("surprise\n");
                    params.result = PluginResult::Err;
                    return;
                }
            };

            let domain = match calldata[1] {
                AbstractCallData::Felt(v) => v,
                _ => {
                    params.result = PluginResult::Err;
                    return;
                }
            };

            let calldata_slice = &calldata[1..(usize::from(domain_length) + 1)];

            match domain_as_str(calldata_slice.iter()) {
                Ok(domain_string) => {
                    starknetid_ctx.domain = domain_string;
                    testing::debug_print("READ DOMAIN: ");
                    testing::debug_print(domain_string.as_str());
                    testing::debug_print("\n");
                    params.result = PluginResult::Ok;
                }
                Err(_) => {
                    testing::debug_print("ERROR: UNABLE TO READ DOMAIN\n");
                    params.result = PluginResult::Err;
                }
            }
        }
        PluginInteractionType::Finalize => {
            testing::debug_print("Finalize plugin\n");
            let value2 = unsafe { *args.add(1) as *mut PluginParam };

            let params: &mut PluginParam = unsafe { &mut *value2 };
            let starknetid_ctx =
                get_context(params.plugin_internal_ctx, params.plugin_internal_ctx_len)
                    .expect("error when getting ctx");

            let data_out: &mut UiParam = unsafe { &mut *(params.data_out as *mut UiParam) };
            data_out.msg.copy_from(&starknetid_ctx.domain);
            params.result = PluginResult::Ok;
        }
        PluginInteractionType::QueryUi => {
            testing::debug_print("QueryUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginParam };

            let params: &mut PluginParam = unsafe { &mut *value2 };
            let out_title = unsafe { &mut *(params.data_out as *mut string::String<32>) };

            let title = "ERC-20 OPERATION".as_bytes();
            out_title.arr[..title.len()].copy_from_slice(title);
            out_title.len = title.len();

            params.result = PluginResult::Ok;
        }
        PluginInteractionType::GetUi => {
            testing::debug_print("GetUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginParam };

            let params: &mut PluginParam = unsafe { &mut *value2 };
            let starknetid_ctx =
                get_context(params.plugin_internal_ctx, params.plugin_internal_ctx_len)
                    .expect("error when getting ctx");
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

        let byte: u8 = if r == ESCAPE {
            if q == FieldElement::ZERO {
                b'a'
            } else {
                return Err(DecodeError::UnsupportedAlphabet);
            }
        } else {
            r.into()
        };

        output.arr[output.len] = byte + if r < LETTERS_LEN { 97u8 } else { 22u8 };
        output.len += 1;
    }
    return Ok(());
}

fn get_context(buf: *mut u8, buf_len: usize) -> Option<&'static mut StarknetIDCtx> {
    let ctx_size = core::mem::size_of::<StarknetIDCtx>();
    let ctx_alignment = core::mem::align_of::<StarknetIDCtx>();
    let buf_addr = buf as usize;
    let offset: isize = (ctx_alignment - (buf_addr % ctx_alignment)) as isize;

    if (buf_len - offset as usize) < ctx_size {
        testing::debug_print("buffer ctx too small!!\n");
        return None;
    }

    let ctx: &mut StarknetIDCtx = unsafe { &mut *(buf.offset(offset) as *mut StarknetIDCtx) };

    Some(ctx)
}
