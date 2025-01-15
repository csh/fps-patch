use std::ffi::CString;

use serde_json::Value;
use winreg::enums::*;
use winreg::RegKey;

fn main() -> anyhow::Result<()> {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let hsr = hklm.open_subkey_with_flags("Software\\Cognosphere\\Star Rail", KEY_ALL_ACCESS)?;

    for (key_name, mut value) in hsr
        .enum_values()
        .map(Result::unwrap)
        .filter(|(key_name, _)| key_name.starts_with("GraphicsSettings_Model_"))
    {
        if value.vtype != RegType::REG_BINARY {
            anyhow::bail!("Incompatible data structure, cancelling execution to avoid corruption.");
        }

        let settings = CString::from_vec_with_nul(value.bytes).expect("read settings");
        let settings =
            serde_json::from_str::<Value>(settings.to_str().unwrap()).expect("json decode");

        if let Value::Object(mut obj) = settings {
            println!("Found settings");

            obj.entry("FPS")
                .and_modify(|value| {
                    if let Value::Number(inner) = value {
                        if inner.as_u64() == Some(120) {
                            println!("120 FPS patch already applied");
                        } else {
                            *value = 120.into();
                            println!("Applying 120 FPS patch");
                        }
                    }
                })
                .or_insert(120.into());

            let settings = CString::new(serde_json::to_string(&obj)?).expect("stringify");
            value.bytes = settings.as_bytes_with_nul().to_vec();

            if let Err(cause) = hsr.set_raw_value(key_name, &value) {
                anyhow::bail!("Failed to write patched settings; {}", cause);
            }
        } else {
            anyhow::bail!("Incompatible data structure, cancelling execution to avoid corruption.");
        }
    }

    Ok(())
}
