import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { check, type Update } from '@tauri-apps/plugin-updater';
import './App.css';

import { RunInfo, BackupInfo } from './types';
import { RunList }      from './components/RunList';
import { BackupList }   from './components/BackupList';
import { SaveDetail }   from './components/SaveDetail';
import { ProfileView }  from './components/ProfileView';

export default function App() {
  const [runs,          setRuns]          = useState<RunInfo[]>([]);
  const [selectedRun,   setSelectedRun]   = useState<RunInfo | null>(null);
  const [backups,       setBackups]       = useState<BackupInfo[]>([]);
  const [selectedBackup,setSelectedBackup]= useState<BackupInfo | null>(null);
  const [image,         setImage]         = useState<string | null>(null);
  const [status,        setStatus]        = useState('');
  const [isError,       setIsError]       = useState(false);
  const [loadingRuns,   setLoadingRuns]   = useState(false);
  const [loadingBackups,setLoadingBackups]= useState(false);
  const [busy,          setBusy]          = useState(false);
  const [saveDirPath,   setSaveDirPath]   = useState<string | null>(null);
  const [backupDirPath, setBackupDirPath] = useState<string>('');

  // ── Updater state ─────────────────────────────────────────────────────────
  const [pendingUpdate,   setPendingUpdate]  = useState<Update | null>(null);
  const [updateBusy,      setUpdateBusy]     = useState(false);
  const [updateDone,      setUpdateDone]     = useState(false);

  // ── Profile editor state ──────────────────────────────────────────────────
  const [profileStep,    setProfileStep]   = useState(0);
  const [profileStatus,  setProfileStatus] = useState('');
  const [profileError,   setProfileError]  = useState(false);
  const [profileBusy,    setProfileBusy]   = useState(false);
  const [profileContent, setProfileContent]= useState<string | null>(null);
  const [centerMode,     setCenterMode]    = useState<'backups' | 'profile'>('backups');

  // ── Helpers ──────────────────────────────────────────────────────────────

  const ok  = (msg: string) => { setStatus(msg); setIsError(false); };
  const err = (msg: string) => { setStatus(msg); setIsError(true);  };

  const pok  = (msg: string) => { setProfileStatus(msg); setProfileError(false); };
  const perr = (msg: string) => { setProfileStatus(msg); setProfileError(true);  };

  const prompt = (message: string, defaultValue = ''): Promise<string | null> =>
    new Promise((resolve) => {
      const value = window.prompt(message, defaultValue);
      resolve(value);
    });

  const confirm = (message: string): boolean => window.confirm(message);

  // ── Data loading ──────────────────────────────────────────────────────────

  const loadRuns = useCallback(async () => {
    setLoadingRuns(true);
    setSelectedRun(null);
    setBackups([]);
    setSelectedBackup(null);
    setImage(null);
    try {
      const result = await invoke<RunInfo[]>('get_honour_saves');
      setRuns(result);
      if (result.length > 0) {
        setSelectedRun(result[0]);
        ok(`Ready.  ${result.length} run${result.length !== 1 ? 's' : ''} found.`);
      } else {
        ok('No HonourMode saves found.');
      }
    } catch (e) {
      err(`Failed to load saves: ${e}`);
    } finally {
      setLoadingRuns(false);
    }
  }, []);

  const loadBackups = useCallback(async (run: RunInfo) => {
    setLoadingBackups(true);
    setSelectedBackup(null);
    setImage(null);
    try {
      const result = await invoke<BackupInfo[]>('get_backups_for_run', {
        saveFolderName: run.folder_name,
      });
      setBackups(result);
    } catch (e) {
      err(`Failed to load backups: ${e}`);
    } finally {
      setLoadingBackups(false);
    }
  }, []);

  const loadImage = useCallback(async (backup: BackupInfo) => {
    setImage(null);
    try {
      const backupDir = await invoke<string>('get_backup_dir_path');
      const backupPath = `${backupDir}\\${backup.folder_name}`;
      const result = await invoke<string | null>('get_backup_image', { backupPath });
      setImage(result);
    } catch {
      setImage(null);
    }
  }, []);

  // ── Initial load ──────────────────────────────────────────────────────────

  useEffect(() => {
    (async () => {
      const sd = await invoke<string | null>('get_save_dir_path').catch(() => null);
      const bd = await invoke<string>('get_backup_dir_path').catch(() => '');
      setSaveDirPath(sd);
      setBackupDirPath(bd);
    })();
    loadRuns();
    // Silent background update check
    check().then(update => { if (update?.available) setPendingUpdate(update); }).catch(() => {});
  }, [loadRuns]);

  // When selected run changes, reload its backups
  useEffect(() => {
    if (selectedRun) loadBackups(selectedRun);
  }, [selectedRun, loadBackups]);

  // When selected backup changes, load its image
  useEffect(() => {
    if (selectedBackup) loadImage(selectedBackup);
    else setImage(null);
  }, [selectedBackup, loadImage]);

  // ── Actions ───────────────────────────────────────────────────────────────

  const handleBackup = async () => {
    if (!selectedRun || busy) return;
    const label = await prompt('Optional label for this backup:', '');
    if (label === null) return; // cancelled
    setBusy(true);
    ok('Creating backup…');
    try {
      const msg = await invoke<string>('backup_save', {
        saveFolder: selectedRun.full_path,
        label: label.trim(),
      });
      ok(msg);
      await loadBackups(selectedRun);
    } catch (e) {
      err(`Backup failed: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handleBackupAll = async () => {
    if (busy) return;
    const label = await prompt('Optional label for this batch backup:', '');
    if (label === null) return;
    setBusy(true);
    ok('Backing up all runs…');
    try {
      const msg = await invoke<string>('backup_all_saves', { label: label.trim() });
      ok(msg);
      if (selectedRun) await loadBackups(selectedRun);
    } catch (e) {
      err(`Backup all failed: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handleRestore = async () => {
    if (!selectedRun || !selectedBackup || busy) return;
    if (!confirm(`Restore "${selectedBackup.folder_name}"?\n\nYour current save will be backed up automatically first.`)) return;
    setBusy(true);
    ok('Restoring…');
    try {
      const msg = await invoke<string>('restore_save', {
        backupName: selectedBackup.folder_name,
        saveName:   selectedRun.folder_name,
      });
      ok(msg);
      await loadBackups(selectedRun);
    } catch (e) {
      err(`Restore failed: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedBackup || busy) return;
    if (!confirm(`Permanently delete "${selectedBackup.folder_name}"?`)) return;
    setBusy(true);
    try {
      await invoke('delete_backup', { backupName: selectedBackup.folder_name });
      ok(`Deleted: ${selectedBackup.folder_name}`);
      setSelectedBackup(null);
      if (selectedRun) await loadBackups(selectedRun);
    } catch (e) {
      err(`Delete failed: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handleOpenSaveDir = async () => {
    if (saveDirPath) await invoke('open_folder', { path: saveDirPath }).catch((e) => err(`Could not open save folder: ${e}`));
  };

  const handleOpenBackupDir = async () => {
    if (backupDirPath) await invoke('open_folder', { path: backupDirPath }).catch((e) => err(`Could not open backup folder: ${e}`));
  };

  // ── Profile editor handlers ───────────────────────────────────────────────

  const handleLoadProfile = async () => {
    setProfileBusy(true);
    pok('Loading profile…');
    try {
      const msg = await invoke<string>('load_profile');
      pok(msg);
      setProfileStep(1);
    } catch (e) { perr(`${e}`); }
    finally { setProfileBusy(false); }
  };

  const handlePrepareProfile = async () => {
    setProfileBusy(true);
    pok('Converting to LSX…');
    try {
      const msg = await invoke<string>('prepare_profile');
      pok(msg);
      setProfileStep(2);
    } catch (e) { perr(`${e}`); }
    finally { setProfileBusy(false); }
  };

  const handleDisplayProfile = async () => {
    setProfileBusy(true);
    pok('Reading profile…');
    try {
      const content = await invoke<string>('get_profile_content');
      setProfileContent(content);
      setCenterMode('profile');
      pok('Displaying profile LSX.');
    } catch (e) { perr(`${e}`); }
    finally { setProfileBusy(false); }
  };

  const handleRemoveFlags = async () => {
    setProfileBusy(true);
    pok('Removing fail flags…');
    try {
      const content = await invoke<string>('remove_fail_flags');
      setProfileContent(content);
      setCenterMode('profile');
      pok('Fail flags removed. Review the changes in the center panel.');
      setProfileStep(3);
    } catch (e) { perr(`${e}`); }
    finally { setProfileBusy(false); }
  };

  const handleSaveProfile = async () => {
    setProfileBusy(true);
    pok('Saving and converting to LSF…');
    try {
      const msg = await invoke<string>('save_profile');
      pok(msg);
      setProfileStep(4);
    } catch (e) { perr(`${e}`); }
    finally { setProfileBusy(false); }
  };

  const handleOverwriteProfile = async () => {
    if (!confirm('Overwrite the live profile8.lsf?\n\nMake sure BG3 is closed before proceeding.')) return;
    setProfileBusy(true);
    pok('Overwriting profile…');
    try {
      const msg = await invoke<string>('overwrite_profile');
      pok(`Done! ${msg}`);
      setProfileStep(5);
    } catch (e) { perr(`${e}`); }
    finally { setProfileBusy(false); }
  };

  // ── Updater handler ───────────────────────────────────────────────────────

  const handleInstallUpdate = async () => {
    if (!pendingUpdate || updateBusy) return;
    setUpdateBusy(true);
    try {
      await pendingUpdate.downloadAndInstall();
      setUpdateDone(true);
    } catch {
      // If install fails silently, dismiss the banner so the user isn't stuck
      setPendingUpdate(null);
    } finally {
      setUpdateBusy(false);
    }
  };

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div className="app">
      <div className="panels">
        <RunList
          runs={runs}
          selected={selectedRun}
          loading={loadingRuns}
          onSelect={setSelectedRun}
          onBackupAll={handleBackupAll}
          profileStep={profileStep}
          profileStatus={profileStatus}
          profileError={profileError}
          profileBusy={profileBusy}
          onLoadProfile={handleLoadProfile}
          onPrepareProfile={handlePrepareProfile}
          onDisplayProfile={handleDisplayProfile}
          onRemoveFlags={handleRemoveFlags}
          onSaveProfile={handleSaveProfile}
          onOverwriteProfile={handleOverwriteProfile}
        />

        {centerMode === 'profile' ? (
          <ProfileView
            content={profileContent}
            onBack={() => setCenterMode('backups')}
          />
        ) : (
          <BackupList
            backups={backups}
            selected={selectedBackup}
            selectedRun={selectedRun}
            loading={loadingBackups}
            onSelect={setSelectedBackup}
            onBackup={handleBackup}
            onRestore={handleRestore}
            onDelete={handleDelete}
            onOpenSaveDir={handleOpenSaveDir}
            onOpenBackupDir={handleOpenBackupDir}
          />
        )}

        {centerMode !== 'profile' && <SaveDetail backup={selectedBackup} image={image} />}
      </div>

      {pendingUpdate && (
        <div className="update-banner">
          <span className="update-banner__text">
            {updateDone
              ? 'Update installed — restart the app to apply it.'
              : `Update available: v${pendingUpdate.version}`}
          </span>
          {!updateDone && (
            <button
              className="update-banner__btn"
              onClick={handleInstallUpdate}
              disabled={updateBusy}
            >
              {updateBusy ? 'Installing…' : 'Install & Restart'}
            </button>
          )}
          <button className="update-banner__dismiss" onClick={() => setPendingUpdate(null)} title="Dismiss">✕</button>
        </div>
      )}

      <button
        className="kofi-btn"
        onClick={() => openUrl('https://ko-fi.com/evernite')}
        title="Support on Ko-fi"
      >
        <img src="/evernite_avatar_sm.png" alt="Evernite" className="kofi-btn__avatar" />
        <span className="kofi-btn__text">
          <span className="kofi-btn__label">Support</span>
          <span className="kofi-btn__name">Evernite</span>
        </span>
        <svg className="kofi-btn__icon" viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
          <path d="M23.881 8.948c-.773-4.085-4.859-4.593-4.859-4.593H.723c-.604 0-.679.798-.679.798s-.082 7.324-.022 11.822c.164 2.424 2.586 2.672 2.586 2.672s8.267-.023 11.966-.049c2.438-.426 2.683-2.566 2.658-3.734 4.352.24 7.422-2.831 6.649-6.916zm-11.062 3.511c-1.246 1.453-4.011 3.976-4.011 3.976s-.121.119-.31.023c-.076-.057-.108-.09-.108-.09-.443-.441-3.368-3.049-4.034-3.954-.709-.965-1.041-2.7-.091-3.71.951-1.01 3.005-1.086 4.363.407 0 0 1.565-1.782 3.468-.963 1.904.82 1.832 2.694.723 4.311zm6.173.478c-.928.116-1.682-.677-1.682-.677V7.418c0-.067.072-.19.19-.19h1.522c.19 0 .19.19.19.19v3.775c.001 0 .437 1.606-.22 1.744z"/>
        </svg>
      </button>

      <div className="bottom-bar">
        <div className={`status-bar${isError ? ' error' : ''}`}>
          {busy && <span className="spinner" />}
          {status}
        </div>
      </div>
    </div>
  );
}
