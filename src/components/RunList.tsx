import { RunInfo } from '../types';
import { ProfileEditor } from './ProfileEditor';

interface Props {
  runs:            RunInfo[];
  selected:        RunInfo | null;
  loading:         boolean;
  onSelect:        (run: RunInfo) => void;
  onBackupAll:     () => void;
  // Profile editor
  profileStep:     number;
  profileStatus:   string;
  profileError:    boolean;
  profileBusy:     boolean;
  onLoadProfile:   () => void;
  onPrepareProfile:() => void;
  onDisplayProfile:() => void;
  onRemoveFlags:   () => void;
  onSaveProfile:   () => void;
  onOverwriteProfile:() => void;
}

export function RunList({
  runs, selected, loading, onSelect, onBackupAll,
  profileStep, profileStatus, profileError, profileBusy,
  onLoadProfile, onPrepareProfile, onDisplayProfile,
  onRemoveFlags, onSaveProfile, onOverwriteProfile,
}: Props) {
  return (
    <div className="panel panel--runs">
      <div className="panel-header">
        <span className="panel-title">Your Runs</span>
        <button
          className="btn-sm"
          onClick={onBackupAll}
          disabled={loading || runs.length === 0}
          title="Backup all runs at once"
        >
          Backup All
        </button>
      </div>

      <div className="list">
        {loading && (
          <div className="list-empty">
            <span className="spinner" />
            Loading saves…
          </div>
        )}

        {!loading && runs.length === 0 && (
          <div className="list-empty">
            No HonourMode saves found.<br />
            Make sure BG3 is installed and you have at least one HonourMode run.
          </div>
        )}

        {!loading && runs.map((run) => {
          const name = run.summary?.display_name ?? run.folder_name;
          const sub  = run.summary
            ? `${run.summary.location}   Party: ${run.summary.party_size}`
            : 'Could not read save info';

          return (
            <div
              key={run.folder_name}
              className={`run-item${selected?.folder_name === run.folder_name ? ' selected' : ''}`}
              onClick={() => onSelect(run)}
            >
              <div className="run-item__name">{name}</div>
              <div className="run-item__sub">{sub}</div>
            </div>
          );
        })}
      </div>

      <ProfileEditor
        step={profileStep}
        status={profileStatus}
        isError={profileError}
        busy={profileBusy}
        onLoad={onLoadProfile}
        onPrepare={onPrepareProfile}
        onDisplay={onDisplayProfile}
        onRemoveFlags={onRemoveFlags}
        onSave={onSaveProfile}
        onOverwrite={onOverwriteProfile}
      />
    </div>
  );
}
