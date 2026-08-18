use jni::objects::{JClass, JString};
use jni::sys::jboolean;
use jni::JNIEnv;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;

mod proxy;

static RUNNING: AtomicBool = AtomicBool::new(false);
static STOP: OnceLock<Mutex<Option<tokio::sync::watch::Sender<bool>>>> = OnceLock::new();

fn stop_cell() -> &'static Mutex<Option<tokio::sync::watch::Sender<bool>>> {
    STOP.get_or_init(|| Mutex::new(None))
}

#[no_mangle]
pub extern "system" fn Java_com_example_bluetoothvpnshare_ProxyService_nativeStart(
    mut env: JNIEnv,
    _class: JClass,
    port: i32,
    user: JString,
    pass: JString,
) -> jboolean {
    if !(1024..=65535).contains(&port) || RUNNING.swap(true, Ordering::SeqCst) {
        return 0;
    }

    let username = env
        .get_string(&user)
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let password = env
        .get_string(&pass)
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (tx, rx) = tokio::sync::watch::channel(false);
    *stop_cell().lock().unwrap() = Some(tx);

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            let _ = rt.block_on(proxy::run(port as u16, username, password, rx));
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
    1
}

#[no_mangle]
pub extern "system" fn Java_com_example_bluetoothvpnshare_ProxyService_nativeStop(
    _env: JNIEnv,
    _class: JClass,
) {
    if let Some(tx) = stop_cell().lock().unwrap().take() {
        let _ = tx.send(true);
    }
    RUNNING.store(false, Ordering::SeqCst);
}
