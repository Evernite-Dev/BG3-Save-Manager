import { BackupInfo, RunInfo } from '../types';

interface Props {
  backups:        BackupInfo[];
  selected:       BackupInfo | null;
  selectedRun:    RunInfo | null;
  loading:        boolean;
  onSelect:       (b: BackupInfo) => void;
  onBackup:       () => void;
  onRestore:      () => void;
  onDelete:       () => void;
  onOpenSaveDir:  () => void;
  onOpenBackupDir: () => void;
}

export function BackupList({
  backups, selected, selectedRun, loading,
  onSelect, onBackup, onRestore, onDelete,
  onOpenSaveDir, onOpenBackupDir,
}: Props) {
  return (
    <div className="panel panel--backups">
      <div className="panel-header">
        <span className="panel-title">Backups for Selected Run</span>
        <span className="panel-count">{backups.length > 0 ? `${backups.length} backup${backups.length !== 1 ? 's' : ''}` : ''}</span>
      </div>

      <div className="list">
        {loading && (
          <div className="list-empty">
            <span className="spinner" />
            Loading backups…
          </div>
        )}

        {!loading && !selectedRun && (
          <div className="list-empty">Select a run to view its backups.</div>
        )}

        {!loading && selectedRun && backups.length === 0 && (
          <div className="list-empty">No backups yet for this run.</div>
        )}

        {!loading && backups.map((b) => (
          <div
            key={b.folder_name}
            className={`backup-item${selected?.folder_name === b.folder_name ? ' selected' : ''}`}
            onClick={() => onSelect(b)}
          >
            <div className="backup-item__top">
              <span className="backup-item__date">{b.date}</span>
              {b.label && <span className="backup-item__label">[{b.label}]</span>}
            </div>
            {b.summary ? (
              <>
                <div className="backup-item__info">{b.summary.display_name}</div>
                <div className="backup-item__location">{b.summary.location}</div>
              </>
            ) : (
              <div className="backup-item__info">{b.folder_name}</div>
            )}
          </div>
        ))}
      </div>

      {/* Action buttons live below the list, above the shared bottom bar */}
      <div style={{ padding: '8px 12px', borderTop: '1px solid var(--border)', display: 'flex', gap: 6, flexWrap: 'wrap', background: 'var(--panel)', flexShrink: 0 }}>
        <button
          className="btn btn--green"
          onClick={onBackup}
          disabled={!selectedRun}
        >
          Backup Run
        </button>
        <button
          className="btn btn--purple"
          onClick={onRestore}
          disabled={!selected}
        >
          Restore Selected
        </button>
        <button
          className="btn btn--red"
          onClick={onDelete}
          disabled={!selected}
        >
          Delete Selected
        </button>
        <span className="btn-spacer" />
        <button className="btn btn--neutral" onClick={onOpenSaveDir}>
          Open Save Folder
        </button>
        <button className="btn btn--neutral" onClick={onOpenBackupDir}>
          Open Backup Folder
        </button>
      </div>
    </div>
  );
}
