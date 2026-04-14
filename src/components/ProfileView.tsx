interface Line {
  text:   string;
  remove: boolean;
}

/** Walk lines tracking XML node depth to find DisabledSingleSaveSessions blocks. */
function classifyLines(content: string): Line[] {
  const lines = content.split('\n');
  const result: Line[] = [];
  let depth = 0;

  for (const text of lines) {
    const t = text.trimStart();

    if (depth === 0) {
      if (t.startsWith('<node id="DisabledSingleSaveSessions"') && !t.trimEnd().endsWith('/>')) {
        // Non-self-closing: actual failure record — start flagging
        depth = 1;
        result.push({ text, remove: true });
      } else {
        result.push({ text, remove: false });
      }
    } else {
      // Inside a target node — everything is flagged for removal
      result.push({ text, remove: true });

      if ((t.startsWith('<node ') || t === '<node>') && !t.includes('/>')) {
        depth++;
      } else if (t.startsWith('</node>')) {
        depth--;
        // depth reaching 0 means the closing tag of the target node was just consumed
      }
    }
  }

  return result;
}

interface Props {
  content: string | null;
  onBack:  () => void;
}

export function ProfileView({ content, onBack }: Props) {
  const lines = content ? classifyLines(content) : null;
  const flagCount = lines ? lines.filter(l => l.remove).length : 0;

  return (
    <div className="panel panel--backups">
      <div className="panel-header">
        <span className="panel-title">Profile LSX</span>
        <span className="profile-view__flag-count">
          {lines && flagCount > 0
            ? `${flagCount} line${flagCount !== 1 ? 's' : ''} flagged for removal`
            : lines
            ? 'No fail flags detected'
            : ''}
        </span>
        <button className="btn-sm" onClick={onBack}>← Backups</button>
      </div>

      <pre className="profile-view__content">
        {lines
          ? lines.map((line, i) => (
              <span
                key={i}
                className={line.remove ? 'profile-line--remove' : undefined}
              >
                {line.text}{'\n'}
              </span>
            ))
          : 'No profile loaded yet.'}
      </pre>
    </div>
  );
}
