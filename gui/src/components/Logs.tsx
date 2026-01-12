import { useState, useEffect } from "react";

interface LogDetails {
  url?: string;
  domain?: string;
  path?: string;
  port?: number;
  method?: string;
  client_ip?: string;
  threat_type?: string;
  reason?: string;
  request_headers?: string[];
}

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  category: string;
  details?: LogDetails;
}

interface LogsProps {
  logs: LogEntry[];
  isDarkMode: boolean;
  initialFilter?: string;
}

function Logs({ logs, isDarkMode, initialFilter }: LogsProps) {
  const [filter, setFilter] = useState<string>(initialFilter || "all");
  const [levelFilter, setLevelFilter] = useState<string>("all");
  const [autoScroll, setAutoScroll] = useState(true);
  const [expandedLog, setExpandedLog] = useState<number | null>(null);

  // Update filter when initialFilter prop changes
  useEffect(() => {
    if (initialFilter) {
      setFilter(initialFilter);
    }
  }, [initialFilter]);

  useEffect(() => {
    // Auto-scroll to bottom if enabled
    if (autoScroll && logs.length > 0) {
      setTimeout(() => {
        const logContainer = document.getElementById("log-container");
        if (logContainer) {
          logContainer.scrollTop = logContainer.scrollHeight;
        }
      }, 100);
    }
  }, [logs, autoScroll]);

  const filteredLogs = logs.filter((log) => {
    // "everything" shows all logs
    // "all" filter shows all blocked/protected traffic (trackers, webrtc, ipv6, security)
    const categoryMatch = filter === "everything"
      ? true
      : filter === "all" 
        ? ["tracker", "webrtc", "ipv6", "security"].includes(log.category)
        : log.category === filter;
    const levelMatch = levelFilter === "all" || log.level === levelFilter;
    return categoryMatch && levelMatch;
  });

  const getLevelColor = (level: string) => {
    if (isDarkMode) {
      switch (level.toLowerCase()) {
        case "error":
          return "text-red-400 bg-red-400/10 border-red-400/30";
        case "warn":
          return "text-yellow-400 bg-yellow-400/10 border-yellow-400/30";
        case "info":
          return "text-blue-400 bg-blue-400/10 border-blue-400/30";
        case "debug":
          return "text-gray-400 bg-gray-400/10 border-gray-400/30";
        default:
          return "text-gray-400 bg-gray-400/10 border-gray-400/30";
      }
    } else {
      switch (level.toLowerCase()) {
        case "error":
          return "text-red-600 bg-red-100 border-red-300";
        case "warn":
          return "text-yellow-700 bg-yellow-100 border-yellow-300";
        case "info":
          return "text-blue-600 bg-blue-100 border-blue-300";
        case "debug":
          return "text-gray-600 bg-gray-100 border-gray-300";
        default:
          return "text-gray-600 bg-gray-100 border-gray-300";
      }
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case "tracker":
        return "⊗";
      case "webrtc":
        return "◉";
      case "ipv6":
        return "⬢";
      case "network":
        return "●";
      case "security":
        return "⚠";
      case "general":
        return "ℹ";
      default:
        return "○";
    }
  };

  const cardClass = isDarkMode
    ? "bg-gray-800/50 backdrop-blur-xl rounded-2xl border border-gray-700/50 p-4"
    : "bg-white/70 backdrop-blur-xl rounded-2xl border border-gray-200/50 p-4";
  
  const textPrimary = isDarkMode ? "text-gray-300" : "text-gray-700";
  const textSecondary = isDarkMode ? "text-gray-400" : "text-gray-600";
  const textTertiary = isDarkMode ? "text-gray-500" : "text-gray-500";
  
  const buttonClass = (isActive: boolean) => isDarkMode
    ? `px-3 py-1.5 rounded-xl text-sm font-medium transition-all duration-200 ${
        isActive
          ? "bg-gradient-to-r from-blue-600 to-cyan-600 text-white shadow-lg shadow-blue-500/30"
          : "bg-gray-700/50 text-gray-400 hover:bg-gray-700 hover:text-white"
      }`
    : `px-3 py-1.5 rounded-xl text-sm font-medium transition-all duration-200 ${
        isActive
          ? "bg-gradient-to-r from-blue-500 to-cyan-500 text-white shadow-lg shadow-blue-500/30"
          : "bg-gray-100 text-gray-600 hover:bg-gray-200 hover:text-gray-900"
      }`;

  const logItemClass = isDarkMode
    ? "flex items-start space-x-3 p-3 bg-gray-700/30 rounded-xl border border-gray-700/50 hover:border-gray-600/50 transition-all duration-200"
    : "flex items-start space-x-3 p-3 bg-gray-50 rounded-xl border border-gray-200/50 hover:border-gray-300/50 transition-all duration-200";

  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className={cardClass}>
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div className="flex items-center space-x-4">
            <label className={`text-sm font-medium ${textPrimary}`}>Category:</label>
            <div className="flex space-x-2 flex-wrap">
              <button
                onClick={() => setFilter("everything")}
                className={buttonClass(filter === "everything")}
              >
                All Logs
              </button>
              <button
                onClick={() => setFilter("all")}
                className={buttonClass(filter === "all")}
              >
                Protected
              </button>
              {["network", "tracker", "webrtc", "ipv6", "security", "general"].map((cat) => (
                <button
                  key={cat}
                  onClick={() => setFilter(cat)}
                  className={buttonClass(filter === cat)}
                >
                  {cat.charAt(0).toUpperCase() + cat.slice(1)}
                </button>
              ))}
            </div>
          </div>

          <div className="flex items-center space-x-4">
            <label className={`text-sm font-medium ${textPrimary}`}>Level:</label>
            <div className="flex space-x-2">
              {["all", "info", "warn", "error", "debug"].map((lev) => (
                <button
                  key={lev}
                  onClick={() => setLevelFilter(lev)}
                  className={buttonClass(levelFilter === lev)}
                >
                  {lev.charAt(0).toUpperCase() + lev.slice(1)}
                </button>
              ))}
            </div>
          </div>

          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="form-checkbox h-4 w-4 text-blue-600 rounded"
            />
            <span className={`text-sm font-medium ${textPrimary}`}>Auto-scroll</span>
          </label>
        </div>
      </div>

      {/* Logs Container */}
      <div
        id="log-container"
        className={`${cardClass} h-[600px] overflow-y-auto space-y-2`}
      >
        {filteredLogs.length === 0 ? (
          <div className={`flex items-center justify-center h-full ${textTertiary}`}>
            <div className="text-center">
              <p>No logs to display</p>
              <p className="text-sm mt-1">Logs will appear here as activity occurs</p>
            </div>
          </div>
        ) : (
          filteredLogs.map((log, index) => (
            <div key={index}>
              <div
                className={`${logItemClass} ${log.details ? 'cursor-pointer hover:border-blue-500/50' : ''}`}
                onClick={() => log.details && setExpandedLog(expandedLog === index ? null : index)}
                title={log.details ? "Click to view details" : ""}
              >
                <div className="text-2xl">{getCategoryIcon(log.category)}</div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center space-x-2 mb-1">
                    <span
                      className={`px-2 py-0.5 rounded-lg text-xs font-medium border ${getLevelColor(
                        log.level
                      )}`}
                    >
                      {log.level.toUpperCase()}
                    </span>
                    <span className={`text-xs ${textSecondary}`}>{log.timestamp}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-lg ${isDarkMode ? 'bg-gray-700 text-gray-400' : 'bg-gray-200 text-gray-600'}`}>
                      {log.category}
                    </span>
                    {log.details && (
                      <span className={`text-xs ${isDarkMode ? 'text-blue-400' : 'text-blue-600'}`}>
                        🔍 {expandedLog === index ? 'Hide' : 'Details'}
                      </span>
                    )}
                  </div>
                  <p className={`text-sm ${textPrimary} break-words`}>{log.message}</p>
                </div>
              </div>
              
              {/* Expanded Details Panel */}
              {expandedLog === index && log.details && (
                <div className={`ml-10 mt-2 p-4 rounded-xl border ${isDarkMode ? 'bg-gray-700/30 border-gray-600' : 'bg-gray-100 border-gray-300'}`}>
                  <h4 className={`text-sm font-bold mb-3 ${textPrimary}`}>📋 Request Details</h4>
                  <div className="space-y-2 text-sm">
                    {log.details.threat_type && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Threat Type: </span>
                        <span className={`${isDarkMode ? 'text-red-400' : 'text-red-600'} font-medium`}>{log.details.threat_type}</span>
                      </div>
                    )}
                    {log.details.reason && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Reason: </span>
                        <span className={textPrimary}>{log.details.reason}</span>
                      </div>
                    )}
                    {log.details.domain && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Domain: </span>
                        <span className={`font-mono ${isDarkMode ? 'text-cyan-400' : 'text-cyan-600'}`}>{log.details.domain}</span>
                      </div>
                    )}
                    {log.details.path && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Path: </span>
                        <span className={`font-mono ${textPrimary}`}>{log.details.path}</span>
                      </div>
                    )}
                    {log.details.port && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Port: </span>
                        <span className={`font-mono ${textPrimary}`}>{log.details.port}</span>
                      </div>
                    )}
                    {log.details.method && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Method: </span>
                        <span className={`font-mono ${isDarkMode ? 'text-green-400' : 'text-green-600'}`}>{log.details.method}</span>
                      </div>
                    )}
                    {log.details.url && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Full URL: </span>
                        <div className={`font-mono text-xs mt-1 p-2 rounded ${isDarkMode ? 'bg-gray-800/50' : 'bg-white'} break-all ${textPrimary}`}>
                          {log.details.url}
                        </div>
                      </div>
                    )}
                    {log.details.client_ip && (
                      <div>
                        <span className={`font-semibold ${textSecondary}`}>Client IP: </span>
                        <span className={`font-mono ${textPrimary}`}>{log.details.client_ip}</span>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>

      {/* Stats */}
      <div className={cardClass}>
        <div className="flex items-center justify-between text-sm">
          <span className={textSecondary}>
            Showing {filteredLogs.length} of {logs.length} logs
          </span>
          {logs.length > 0 && (
            <span className={textSecondary}>
              Last updated: {logs[logs.length - 1]?.timestamp}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

export default Logs;
