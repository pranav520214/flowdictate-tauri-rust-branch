use std::path::PathBuf;
use tauri::{App, Manager};
use tracing::{info, warn};

/// Runs the first-launch setup tasks (hardware detection, model download, integrity check).
pub fn run_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");

    let models_dir = app_data_dir.join("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)?;
    }

    // Hardware detection
    let has_cuda = detect_cuda();
    if has_cuda {
        info!("CUDA compatible hardware detected. Will configure ONNX Runtime for CUDA execution provider.");
    } else {
        info!("No CUDA compatible hardware detected. Defaulting to CPU execution provider.");
    }

    // Kick off background model download if missing
    tauri::async_runtime::spawn(async move {
        if let Err(e) = download_models_if_missing(models_dir).await {
            warn!("Model download or verification failed: {}", e);
        }
    });

    Ok(())
}

/// Rudimentary CUDA detection (e.g. checking for nvml or nvcuda.dll)
fn detect_cuda() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Simple heuristic: check for nvcuda.dll
        std::path::Path::new("C:\\Windows\\System32\\nvcuda.dll").exists()
    }
    #[cfg(not(target_os = "windows"))]
    {
        // On Linux, check for libcuda.so or similar, or nvidia-smi
        std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcuda.so").exists() ||
        std::path::Path::new("/usr/bin/nvidia-smi").exists()
    }
}

async fn download_models_if_missing(models_dir: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Example URLs (placeholder for actual HuggingFace Hub links)
    let asr_model_url = "https://huggingface.co/nvidia/Nemotron-3-8B-Base-4k/resolve/main/nemotron.onnx";
    let llm_model_url = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf";

    let asr_path = models_dir.join("nemotron.onnx");
    let llm_path = models_dir.join("qwen2.5-0.5b-instruct-q4_k_m.gguf");

    if !asr_path.exists() {
        info!("Downloading Nemotron ASR model...");
        download_file(asr_model_url, &asr_path).await?;
        info!("Nemotron ASR model downloaded successfully.");
    }

    if !llm_path.exists() {
        info!("Downloading Qwen2.5-0.5B purification model...");
        download_file(llm_model_url, &llm_path).await?;
        info!("Purification model downloaded successfully.");
    }

    // Integrity checks would happen here or in the core logic before loading
    Ok(())
}

async fn download_file(url: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await?.error_for_status()?;
    
    let mut file = std::fs::File::create(path)?;
    while let Some(chunk) = response.chunk().await? {
        use std::io::Write;
        file.write_all(&chunk)?;
    }
    
    Ok(())
}
