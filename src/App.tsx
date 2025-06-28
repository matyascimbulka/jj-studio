import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, GitBranch, User, Clock, Hash } from "lucide-react";
import "./App.css";

interface JJChange {
  change_id: string;
  commit_id: string;
  description: string;
  author: string;
  timestamp: string;
}

function App() {
  const [repoPath, setRepoPath] = useState<string>("");
  const [changes, setChanges] = useState<JJChange[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");

  async function selectRepository() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select JJ Repository Directory",
      });

      if (selected) {
        setLoading(true);
        setError("");
        
        // Validate if it's a JJ repository
        const isValid = await invoke<boolean>("validate_jj_repo", { path: selected });
        
        if (isValid) {
          setRepoPath(selected);
          await loadChanges(selected);
        }
      }
    } catch (err) {
      setError(err as string);
      setLoading(false);
    }
  }

  async function loadChanges(path: string) {
    try {
      setLoading(true);
      const jjChanges = await invoke<JJChange[]>("get_jj_changes", { path });
      setChanges(jjChanges);
      setError("");
    } catch (err) {
      setError(err as string);
      setChanges([]);
    } finally {
      setLoading(false);
    }
  }

  function formatTimestamp(timestamp: string): string {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return timestamp;
    }
  }

  function truncateId(id: string, length: number = 12): string {
    return id.length > length ? id.substring(0, length) : id;
  }

  return (
    <div className="app">
      <header className="header">
        <h1>JJ Studio</h1>
        <div className="repo-selector">
          {repoPath && (
            <span className="repo-path" title={repoPath}>
              {repoPath}
            </span>
          )}
          <button className="select-repo-btn" onClick={selectRepository}>
            <FolderOpen size={16} />
            {repoPath ? "Change Repository" : "Select Repository"}
          </button>
        </div>
      </header>

      <main className="main-content">
        {!repoPath && !loading && !error && (
          <div className="welcome-screen">
            <h2>Welcome to JJ Studio</h2>
            <p>
              A beautiful GUI for Jujutsu VCS. Select a repository directory to get started
              and explore your changes with a modern, intuitive interface.
            </p>
            <button className="select-repo-btn" onClick={selectRepository}>
              <FolderOpen size={20} />
              Select JJ Repository
            </button>
          </div>
        )}

        {loading && (
          <div className="loading">
            <GitBranch size={24} />
            <span style={{ marginLeft: "0.5rem" }}>Loading changes...</span>
          </div>
        )}

        {error && (
          <div className="error">
            <h3>Error</h3>
            <p>{error}</p>
          </div>
        )}

        {repoPath && !loading && !error && changes.length > 0 && (
          <div className="changes-container">
            <div className="changes-header">
              <h2>Recent Changes</h2>
            </div>
            <div className="changes-list">
              {changes.map((change, index) => (
                <div key={`${change.change_id}-${index}`} className="change-item">
                  <div className="change-item-header">
                    <h3 className="change-description">{change.description}</h3>
                    <span className="change-timestamp">
                      <Clock size={12} />
                      {formatTimestamp(change.timestamp)}
                    </span>
                  </div>
                  <div className="change-meta">
                    <span className="change-id">
                      <Hash size={12} />
                      Change: {truncateId(change.change_id)}
                    </span>
                    <span className="change-id">
                      <GitBranch size={12} />
                      Commit: {truncateId(change.commit_id)}
                    </span>
                    <span className="change-author">
                      <User size={12} />
                      {change.author}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {repoPath && !loading && !error && changes.length === 0 && (
          <div className="welcome-screen">
            <h2>No Changes Found</h2>
            <p>This repository doesn't have any changes to display.</p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
