export interface SaveSummary {
  display_name: string;
  location:     string;
  companions:   string;
  level:        number;
  classes:      string;
  race:         string;
  party_size:   number;
}

export interface RunInfo {
  folder_name: string;
  full_path:   string;
  summary:     SaveSummary | null;
}

export interface BackupInfo {
  folder_name: string;
  display:     string;
  label:       string;
  date:        string;
  summary:     SaveSummary | null;
}
