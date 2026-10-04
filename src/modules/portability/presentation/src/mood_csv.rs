use portability_application::models::mood_record::MoodRecord;

pub fn to_csv(entries: &[MoodRecord]) -> Result<Vec<u8>, String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer
        .write_record(["recorded_at", "score", "note"])
        .map_err(|e| e.to_string())?;
    for e in entries {
        let note = e.note.clone().unwrap_or_default();
        let note = match note.starts_with(['=', '+', '-', '@']) {
            true => format!("'{note}"),
            false => note,
        };
        writer
            .write_record(&[e.recorded_at.to_rfc3339(), e.score.to_string(), note])
            .map_err(|e| e.to_string())?;
    }
    writer.into_inner().map_err(|e| e.to_string())
}

pub fn mood_records_from_csv(bytes: &[u8]) -> Result<Vec<MoodRecord>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);
    let mut records = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| format!("not a valid CSV export: {e}"))?;
        if row.len() != 3 {
            return Err(format!("expected 3 columns, found {}", row.len()));
        }
        let recorded = row.get(0).map(str::to_string).unwrap_or_default();
        let score_raw = row.get(1).map(str::to_string).unwrap_or_default();
        let note_raw = row.get(2).map(str::to_string).unwrap_or_default();
        let recorded_at = chrono::DateTime::parse_from_rfc3339(&recorded)
            .map_err(|e| format!("invalid recorded_at '{recorded}': {e}"))?
            .with_timezone(&chrono::Utc);
        let score: u8 = score_raw
            .parse()
            .map_err(|e| format!("invalid score '{score_raw}': {e}"))?;
        let note = match note_raw.strip_prefix('\'') {
            Some(rest) if rest.starts_with(['=', '+', '-', '@']) && !note_raw.starts_with("''") => {
                rest.to_string()
            }
            _ => note_raw,
        };
        let note = (!note.is_empty()).then_some(note);
        records.push(MoodRecord {
            score,
            note,
            recorded_at,
        });
    }
    Ok(records)
}
