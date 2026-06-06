mod crc16;
mod dp100;

#[tauri::command]
fn get_basic_info() -> Result<dp100::BasicInfo, String> {
    let dev = dp100::Dp100::open()?;
    dev.get_basic_info()
    //Ok(dp100::BasicInfo {
    //    vin: 12.0,
    //    vout: 5.0,
    //    iout: 1.0,
    //    temp: 35.0,
    //})
}

#[tauri::command]
fn get_device_info() -> Result<String, String> {
    let dev = dp100::Dp100::open()?;
    dev.get_device_info()
    //Ok("ATK-DP100 (ダミー)".to_string())
}

// 全プロファイル取得（起動時に1回呼ぶ）
#[tauri::command]
fn get_all_profiles() -> Result<Vec<dp100::Profile>, String> {
    let dev = dp100::Dp100::open()?;
    let mut profiles = Vec::new();
    for i in 0..10 {
        profiles.push(dev.get_profile(i)?); // ダミーpush→実機取得
                                            // ダミー実装: 実機接続後は get_profile(i) に差し替え
                                            //profiles.push(dp100::Profile {
                                            //    index: i,
                                            //    output_on: false,
                                            //    vset: 5.0 + i as f64 * 0.5,
                                            //    iset: 1.0,
                                            //    ovp: 33.0,
                                            //    ocp: 5.5,
                                            //});
    }
    //log::info!("get_all_profiles returning {} profiles", profiles.len());
    Ok(profiles)
}

#[tauri::command]
fn set_profile(profile: dp100::Profile) -> Result<(), String> {
    let dev = dp100::Dp100::open()?;
    dev.set_profile(&profile)
}

#[tauri::command]
fn set_output_immediate(
    index: u8,
    on: bool,
    vset: f64,
    iset: f64,
    ovp: f64,
    ocp: f64,
) -> Result<(), String> {
    let dev = dp100::Dp100::open()?;
    dev.set_output_immediate(index, on, vset, iset, ovp, ocp)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_basic_info,
            get_device_info,
            get_all_profiles,
            set_profile,
            set_output_immediate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
