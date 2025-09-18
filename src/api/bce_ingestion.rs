// BCE Record Ingestion API
// Provides HTTP endpoints for receiving BCE records from operator billing systems

use crate::bce_pipeline::{BCERecord, BCEPipeline};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Reply};
use tracing::{info, warn, error};

/// BCE API Server for ingesting records from operator billing systems
pub struct BCEIngestAPI {
    pipeline: Arc<Mutex<BCEPipeline>>,
    port: u16,
}

/// BCE record submission request
#[derive(Debug, Deserialize, Serialize)]
pub struct BCERecordRequest {
    pub record: BCERecord,
    pub operator_signature: Option<String>, // Optional BLS signature from operator
}

/// API Response for BCE record submission
#[derive(Debug, Serialize)]
pub struct BCEResponse {
    pub success: bool,
    pub message: String,
    pub batch_id: Option<String>,
}

/// Batch processing status
#[derive(Debug, Serialize)]
pub struct BatchStatus {
    pub batch_id: String,
    pub record_count: usize,
    pub total_charges_cents: u64,
    pub processing_status: String,
}

impl BCEIngestAPI {
    pub fn new(pipeline: Arc<Mutex<BCEPipeline>>, port: u16) -> Self {
        Self { pipeline, port }
    }

    /// Start the BCE ingestion API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🌐 Starting BCE Record Ingestion API on port {}", self.port);

        let pipeline = self.pipeline.clone();

        // POST /api/v1/bce/submit - Submit individual BCE record
        let submit_record = warp::path!("api" / "v1" / "bce" / "submit")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_pipeline(pipeline.clone()))
            .and_then(submit_bce_record);

        // GET /api/v1/bce/batch/{batch_id}/status - Check batch status
        let batch_status = warp::path!("api" / "v1" / "bce" / "batch" / String / "status")
            .and(warp::get())
            .and(with_pipeline(pipeline.clone()))
            .and_then(get_batch_status);

        // POST /api/v1/bce/batch/submit - Submit multiple BCE records
        let batch_submit = warp::path!("api" / "v1" / "bce" / "batch" / "submit")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_pipeline(pipeline.clone()))
            .and_then(submit_bce_batch);

        // GET /api/v1/bce/stats - Get pipeline statistics
        let stats = warp::path!("api" / "v1" / "bce" / "stats")
            .and(warp::get())
            .and(with_pipeline(pipeline.clone()))
            .and_then(get_pipeline_stats);

        // Health check endpoint
        let health = warp::path!("health")
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({"status": "healthy", "service": "SP-BCE-Ingestion"})));

        let routes = submit_record
            .or(batch_status)
            .or(batch_submit)
            .or(stats)
            .or(health)
            .with(warp::cors().allow_any_origin().allow_headers(vec!["content-type"]).allow_methods(vec!["GET", "POST"]));

        info!("✅ BCE API ready - accepting BCE records from operator billing systems");
        info!("📡 Endpoints:");
        info!("   POST /api/v1/bce/submit - Submit single BCE record");
        info!("   POST /api/v1/bce/batch/submit - Submit BCE record batch");
        info!("   GET  /api/v1/bce/batch/{{batch_id}}/status - Check batch status");
        info!("   GET  /api/v1/bce/stats - Pipeline statistics");
        info!("   GET  /health - Health check");

        warp::serve(routes)
            .run(([0, 0, 0, 0], self.port))
            .await;

        Ok(())
    }
}

/// Submit single BCE record
async fn submit_bce_record(
    request: BCERecordRequest,
    pipeline: Arc<Mutex<BCEPipeline>>
) -> Result<impl Reply, warp::Rejection> {
    info!("📋 Received BCE record: {} from PLMN {}->{}",
          request.record.record_id,
          request.record.home_plmn,
          request.record.visited_plmn);

    let mut pipeline = pipeline.lock().await;

    match pipeline.process_bce_record(request.record.clone()).await {
        Ok(()) => {
            let response = BCEResponse {
                success: true,
                message: format!("BCE record {} processed successfully", request.record.record_id),
                batch_id: Some(format!("batch_{}_{}", request.record.home_plmn, request.record.visited_plmn)),
            };

            info!("✅ BCE record processed: {}", request.record.record_id);
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            error!("❌ Failed to process BCE record {}: {:?}", request.record.record_id, e);
            let response = BCEResponse {
                success: false,
                message: format!("Failed to process BCE record: {}", e),
                batch_id: None,
            };
            Ok(warp::reply::json(&response))
        }
    }
}

/// Submit batch of BCE records
async fn submit_bce_batch(
    records: Vec<BCERecordRequest>,
    pipeline: Arc<Mutex<BCEPipeline>>
) -> Result<impl Reply, warp::Rejection> {
    info!("📦 Received BCE batch with {} records", records.len());

    let mut pipeline = pipeline.lock().await;
    let mut successful = 0;
    let mut failed = 0;

    for record_request in records {
        match pipeline.process_bce_record(record_request.record.clone()).await {
            Ok(()) => successful += 1,
            Err(e) => {
                warn!("Failed to process BCE record {}: {:?}", record_request.record.record_id, e);
                failed += 1;
            }
        }
    }

    let response = BCEResponse {
        success: failed == 0,
        message: format!("Processed {} records successfully, {} failed", successful, failed),
        batch_id: Some(format!("batch_{}", chrono::Utc::now().timestamp())),
    };

    info!("✅ BCE batch processed: {} successful, {} failed", successful, failed);
    Ok(warp::reply::json(&response))
}

/// Get batch processing status
async fn get_batch_status(
    batch_id: String,
    pipeline: Arc<Mutex<BCEPipeline>>
) -> Result<impl Reply, warp::Rejection> {
    let pipeline = pipeline.lock().await;

    // Mock batch status - in real implementation would track actual batches
    let status = BatchStatus {
        batch_id: batch_id.clone(),
        record_count: 0,
        total_charges_cents: 0,
        processing_status: "completed".to_string(),
    };

    Ok(warp::reply::json(&status))
}

/// Get pipeline statistics
async fn get_pipeline_stats(
    pipeline: Arc<Mutex<BCEPipeline>>
) -> Result<impl Reply, warp::Rejection> {
    let pipeline = pipeline.lock().await;
    let stats = pipeline.get_stats();

    Ok(warp::reply::json(stats))
}

/// Warp filter to pass pipeline to handlers
fn with_pipeline(
    pipeline: Arc<Mutex<BCEPipeline>>
) -> impl Filter<Extract = (Arc<Mutex<BCEPipeline>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pipeline.clone())
}

/// Example curl commands for testing
pub fn print_curl_examples(port: u16) {
    println!("📡 BCE API Curl Examples:");
    println!("");

    println!("1️⃣ Submit T-Mobile Germany -> Vodafone UK data session:");
    println!("curl -X POST http://localhost:{}/api/v1/bce/submit \\", port);
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"record\": {{");
    println!("      \"record_id\": \"BCE_20240318_TMO_DE_001247856\",");
    println!("      \"record_type\": \"DATA_SESSION_CDR\",");
    println!("      \"imsi\": \"262011234567890\",");
    println!("      \"home_plmn\": \"26201\",");
    println!("      \"visited_plmn\": \"23410\",");
    println!("      \"session_duration\": 213,");
    println!("      \"bytes_uplink\": 1247680,");
    println!("      \"bytes_downlink\": 8932456,");
    println!("      \"wholesale_charge\": 23822,");
    println!("      \"retail_charge\": 31250,");
    println!("      \"currency\": \"EUR\",");
    println!("      \"timestamp\": {},", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    println!("      \"charging_id\": 987654321");
    println!("    }}");
    println!("  }}'");
    println!("");

    println!("2️⃣ Submit Orange France -> Vodafone UK voice call:");
    println!("curl -X POST http://localhost:{}/api/v1/bce/submit \\", port);
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"record\": {{");
    println!("      \"record_id\": \"BCE_20240318_ORG_FR_002156789\",");
    println!("      \"record_type\": \"VOICE_CALL_CDR\",");
    println!("      \"imsi\": \"208011234567890\",");
    println!("      \"home_plmn\": \"20801\",");
    println!("      \"visited_plmn\": \"23415\",");
    println!("      \"session_duration\": 347,");
    println!("      \"bytes_uplink\": 0,");
    println!("      \"bytes_downlink\": 0,");
    println!("      \"wholesale_charge\": 18020,");
    println!("      \"retail_charge\": 26015,");
    println!("      \"currency\": \"EUR\",");
    println!("      \"timestamp\": {},", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    println!("      \"charging_id\": 987654322");
    println!("    }}");
    println!("  }}'");
    println!("");

    println!("3️⃣ Check pipeline statistics:");
    println!("curl http://localhost:{}/api/v1/bce/stats", port);
    println!("");

    println!("4️⃣ Health check:");
    println!("curl http://localhost:{}/health", port);
    println!("");
}