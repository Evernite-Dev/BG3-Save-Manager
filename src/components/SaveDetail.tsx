import { BackupInfo } from '../types';

interface Props {
  backup:  BackupInfo | null;
  image:   string | null;   // base64 WebP
}

export function SaveDetail({ backup, image }: Props) {
  const s = backup?.summary ?? null;

  return (
    <div className="panel panel--detail detail">
      <div className="panel-header">
        <span className="panel-title">Snapshot</span>
      </div>

      {/* Screenshot */}
      <div className="snapshot-wrap">
        {image ? (
          <img
            src={`data:image/webp;base64,${image}`}
            alt="Save snapshot"
            draggable={false}
          />
        ) : (
          <span className="snapshot-placeholder">No snapshot</span>
        )}
      </div>

      {/* Character / save details */}
      <div className="detail-info">
        {!backup && (
          <div style={{ color: 'var(--text-faint)', fontSize: 12 }}>
            Select a backup to view details.
          </div>
        )}

        {backup && (
          <>
            {backup.label && (
              <span className="detail-note">{backup.label}</span>
            )}

            <div className="detail-row">
              <div className="detail-label">Backed Up</div>
              <div className="detail-value" style={{ fontFamily: 'var(--font-mono)', fontSize: 11 }}>
                {backup.date}
              </div>
            </div>

            {s && (
              <>
                <div className="detail-divider" />

                <div className="detail-row">
                  <div className="detail-label">Character</div>
                  <div className="detail-value">{s.race}</div>
                </div>

                <div className="detail-row">
                  <div className="detail-label">Class / Level</div>
                  <div className="detail-value">{s.classes}  —  Level {s.level}</div>
                </div>

                <div className="detail-row">
                  <div className="detail-label">Location</div>
                  <div className="detail-value">{s.location}</div>
                </div>

                <div className="detail-divider" />

                <div className="detail-row">
                  <div className="detail-label">Party Size</div>
                  <div className="detail-value">{s.party_size}</div>
                </div>

                {s.companions && (
                  <div className="detail-row">
                    <div className="detail-label">Companions</div>
                    <div className="detail-value" style={{ lineHeight: 1.6 }}>
                      {s.companions.split(', ').map((c, i) => (
                        <div key={i}>{c}</div>
                      ))}
                    </div>
                  </div>
                )}
              </>
            )}
          </>
        )}
      </div>
    </div>
  );
}
