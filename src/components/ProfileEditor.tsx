interface Props {
  step:            number;   // 0=idle 1=loaded 2=prepared 3=flagsRemoved 4=saved 5=done
  status:          string;
  isError:         boolean;
  busy:            boolean;
  onLoad:          () => void;
  onPrepare:       () => void;
  onDisplay:       () => void;
  onRemoveFlags:   () => void;
  onSave:          () => void;
  onOverwrite:     () => void;
}

export function ProfileEditor({
  step, status, isError, busy,
  onLoad, onPrepare, onDisplay, onRemoveFlags, onSave, onOverwrite,
}: Props) {
  return (
    <div className="profile-editor">
      <div className="profile-editor__header">
        <span className="panel-title">Profile Editor</span>
        <span className="profile-editor__hint">Clear failed Honour Modes</span>
      </div>

      <div className="profile-editor__grid">
        <button
          className="btn btn--neutral profile-editor__btn"
          onClick={onLoad}
          disabled={busy}
          title="Locate and back up profile8.lsf"
        >
          1. Load Profile
        </button>
        <button
          className="btn btn--neutral profile-editor__btn"
          onClick={onPrepare}
          disabled={busy || step < 1}
          title="Convert profile8.lsf to editable LSX"
        >
          2. Prepare Profile
        </button>
        <button
          className="btn btn--neutral profile-editor__btn"
          onClick={onDisplay}
          disabled={busy || step < 2}
          title="View the LSX file in the center panel"
        >
          3. Display Profile
        </button>
        <button
          className="btn btn--purple profile-editor__btn"
          onClick={onRemoveFlags}
          disabled={busy || step < 2}
          title="Remove DisabledSingleSaveSessions nodes"
        >
          4. Remove Fail Flags
        </button>
        <button
          className="btn btn--green profile-editor__btn"
          onClick={onSave}
          disabled={busy || step < 3}
          title="Convert edited LSX back to LSF"
        >
          5. Save Edited File
        </button>
        <button
          className="btn btn--red profile-editor__btn"
          onClick={onOverwrite}
          disabled={busy || step < 4}
          title="Overwrite the live profile8.lsf with the edited version"
        >
          6. Overwrite Profile
        </button>
      </div>

      {status && (
        <div className={`profile-editor__status${isError ? ' error' : ''}`}>
          {busy && <span className="spinner" />}
          {status}
        </div>
      )}
    </div>
  );
}
