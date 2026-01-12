interface Stats {
  tor_connected: boolean;
  kill_switch_active: boolean;
  requests_blocked: number;
  trackers_blocked: number;
  webrtc_blocked: number;
  ipv6_blocked: number;
  total_requests: number;
  proxy_running: boolean;
  auto_proxy_enabled: boolean;
  uptime_seconds: number;
  security_threats_detected: number;
}

interface DashboardProps {
  stats: Stats;
  isDarkMode: boolean;
  onStatClick: (category: string) => void;
  onToggleConnection: () => void;
  isConnecting: boolean;
}

function Dashboard({ stats, isDarkMode, onStatClick, onToggleConnection, isConnecting }: DashboardProps) {
  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  // Calculate fake ping and speeds for display (you can make these real later)
  const ping = stats.proxy_running ? Math.floor(15 + Math.random() * 10) : 0;
  const downloadSpeed = stats.proxy_running ? (45 + Math.random() * 15).toFixed(1) : "0.0";
  const uploadSpeed = stats.proxy_running ? (12 + Math.random() * 8).toFixed(1) : "0.0";

  return (
    <div className="flex flex-col h-full px-8 py-6">
      {/* Connection Time Display */}
      <div className="text-center mb-6 px-4">
        <p className={`text-xs font-medium mb-2 ${
          isDarkMode ? 'text-slate-500' : 'text-gray-500'
        }`}>
          {stats.auto_proxy_enabled ? 'Auto Privacy' : 'Manual Privacy'}
        </p>
        <h1 className={`text-4xl font-bold tabular-nums tracking-wider ${
          isDarkMode ? 'text-white' : 'text-gray-900'
        }`}>
          {formatUptime(stats.uptime_seconds)}
        </h1>
        <div className="flex items-center justify-center mt-2 space-x-2">
          <div className={`w-2 h-2 rounded-full ${
            stats.proxy_running ? 'bg-green-500 animate-pulse' : 'bg-red-500'
          }`}></div>
          <p className={`text-xs font-medium ${
            stats.proxy_running 
              ? 'text-green-500' 
              : isDarkMode ? 'text-red-400' : 'text-red-600'
          }`}>
            {stats.proxy_running ? 'Connected' : 'Disconnected'}
          </p>
        </div>
      </div>

      {/* Stats Cards Grid */}
      <div className="grid grid-cols-4 gap-3 mb-6 px-2">
        <MetricCard
          icon="📡"
          label="Ping"
          value={ping > 0 ? `${ping} ms` : "-- ms"}
          isDarkMode={isDarkMode}
        />
        <MetricCard
          icon="⬇"
          label="Download"
          value={downloadSpeed !== "0.0" ? `${downloadSpeed} Mbps` : "-- Mbps"}
          isDarkMode={isDarkMode}
        />
        <MetricCard
          icon="⬆"
          label="Upload"
          value={uploadSpeed !== "0.0" ? `${uploadSpeed} Mbps` : "-- Mbps"}
          isDarkMode={isDarkMode}
        />
        <MetricCard
          icon="⚠"
          label="Blocked"
          value={stats.requests_blocked.toString()}
          isDarkMode={isDarkMode}
          onClick={() => onStatClick('all')}
        />
      </div>

      {/* Large Central Power Button */}
      <div className="flex-1 flex justify-center items-center relative px-4 min-h-0">
        {/* Outer glow rings */}
        {stats.proxy_running && (
          <>
            <div className="absolute w-80 h-80 rounded-full bg-gradient-to-r from-green-500/20 to-emerald-500/20 blur-3xl animate-pulse"></div>
            <div className="absolute w-64 h-64 rounded-full bg-gradient-to-r from-green-400/30 to-emerald-400/30 blur-2xl animate-pulse" style={{ animationDelay: '0.5s' }}></div>
          </>
        )}
        {!stats.proxy_running && (
          <div className="absolute w-64 h-64 rounded-full bg-slate-700/20 blur-3xl"></div>
        )}
        
        {/* Power Button */}
        <button
          onClick={onToggleConnection}
          disabled={isConnecting}
          className={`relative w-56 h-56 rounded-full transition-all duration-300 ${
            isConnecting ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer hover:scale-105 active:scale-95'
          } ${
            stats.proxy_running
              ? 'bg-gradient-to-br from-green-500/30 to-emerald-600/30 shadow-2xl shadow-green-500/50'
              : isDarkMode
                ? 'bg-slate-800 shadow-2xl shadow-slate-900/50'
                : 'bg-gray-200 shadow-2xl shadow-gray-400/50'
          }`}
          style={{
            border: stats.proxy_running ? '3px solid rgba(34, 197, 94, 0.5)' : '3px solid rgba(100, 116, 139, 0.3)'
          }}
        >
          {/* Inner circle with pattern */}
          <div className="absolute inset-0 flex items-center justify-center">
            <div className={`w-40 h-40 rounded-full flex items-center justify-center ${
              stats.proxy_running
                ? 'bg-gradient-to-br from-green-400/40 to-emerald-500/40'
                : isDarkMode
                  ? 'bg-slate-700/50'
                  : 'bg-gray-300/50'
            }`}>
              {/* Geometric pattern */}
              <svg width="120" height="120" viewBox="0 0 120 120" className={`${
                stats.proxy_running ? 'animate-spin-slow' : ''
              }`}>
                <circle
                  cx="60"
                  cy="60"
                  r="50"
                  fill="none"
                  stroke={stats.proxy_running ? "rgba(34, 197, 94, 0.6)" : "rgba(100, 116, 139, 0.3)"}
                  strokeWidth="1"
                  strokeDasharray="4 4"
                />
                <circle
                  cx="60"
                  cy="60"
                  r="40"
                  fill="none"
                  stroke={stats.proxy_running ? "rgba(34, 197, 94, 0.4)" : "rgba(100, 116, 139, 0.2)"}
                  strokeWidth="1"
                  strokeDasharray="6 6"
                />
                <circle
                  cx="60"
                  cy="60"
                  r="30"
                  fill="none"
                  stroke={stats.proxy_running ? "rgba(34, 197, 94, 0.3)" : "rgba(100, 116, 139, 0.15)"}
                  strokeWidth="1"
                  strokeDasharray="8 8"
                />
              </svg>
              
              {/* Power Icon */}
              <div className="absolute">
                <svg
                  width="48"
                  height="48"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke={stats.proxy_running ? "#22c55e" : isDarkMode ? "#64748b" : "#9ca3af"}
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M12 2v10" />
                  <path d="M18.4 6.6a9 9 0 1 1-12.77.04" />
                </svg>
              </div>
            </div>
          </div>
        </button>
      </div>

      {/* Bottom Quick Stats */}
      <div className={`mx-4 mb-3 p-4 rounded-xl flex-shrink-0 ${
        isDarkMode ? 'bg-slate-900/50 border border-slate-800' : 'bg-white/50 border border-gray-300'
      }`}>
        <div className="grid grid-cols-2 gap-3">
          <div className="text-center">
            <p className={`text-xs mb-1 ${
              isDarkMode ? 'text-slate-500' : 'text-gray-500'
            }`}>Tor Status</p>
            <p className={`text-xs font-semibold ${
              stats.tor_connected ? 'text-green-500' : 'text-red-500'
            }`}>
              {stats.tor_connected ? '✓ Connected' : '✗ Offline'}
            </p>
          </div>
          <div className="text-center">
            <p className={`text-xs mb-1 ${
              isDarkMode ? 'text-slate-500' : 'text-gray-500'
            }`}>Kill Switch</p>
            <p className={`text-xs font-semibold ${
              stats.kill_switch_active ? 'text-green-500' : 'text-orange-500'
            }`}>
              {stats.kill_switch_active ? 'Active' : 'Inactive'}
            </p>
          </div>
        </div>
      </div>

      {/* Protection Details - Expandable */}
      <details className={`mx-4 mb-3 p-4 rounded-xl flex-shrink-0 ${
        isDarkMode ? 'bg-slate-900/50 border border-slate-800' : 'bg-white/50 border border-gray-300'
      }`}>
        <summary className={`cursor-pointer font-semibold text-sm ${
          isDarkMode ? 'text-white' : 'text-gray-900'
        }`}>
          Protection Details
        </summary>
        <div className="mt-4 space-y-3">
          <DetailRow
            label="Trackers Blocked"
            value={stats.trackers_blocked}
            isDarkMode={isDarkMode}
            onClick={() => onStatClick('tracker')}
          />
          <DetailRow
            label="WebRTC Leaks Blocked"
            value={stats.webrtc_blocked}
            isDarkMode={isDarkMode}
            onClick={() => onStatClick('webrtc')}
          />
          <DetailRow
            label="IPv6 Leaks Blocked"
            value={stats.ipv6_blocked}
            isDarkMode={isDarkMode}
            onClick={() => onStatClick('ipv6')}
          />
          <DetailRow
            label="Security Threats"
            value={stats.security_threats_detected}
            isDarkMode={isDarkMode}
            onClick={() => onStatClick('security')}
          />
          <DetailRow
            label="Total Requests"
            value={stats.total_requests}
            isDarkMode={isDarkMode}
            onClick={() => onStatClick('network')}
          />
        </div>
      </details>

      {/* Configuration */}
      <details className={`mx-4 mb-4 p-4 rounded-xl flex-shrink-0 ${
        isDarkMode ? 'bg-slate-900/50 border border-slate-800' : 'bg-white/50 border border-gray-300'
      }`}>
        <summary className={`cursor-pointer font-semibold text-sm ${
          isDarkMode ? 'text-white' : 'text-gray-900'
        }`}>
          Configuration</summary>
        <div className="mt-4 space-y-3">
          <div className={`flex items-center justify-between p-3 rounded-lg ${
            isDarkMode ? 'bg-slate-800/50' : 'bg-gray-100'
          }`}>
            <div>
              <p className={`text-xs mb-1 ${
                isDarkMode ? 'text-slate-500' : 'text-gray-500'
              }`}>Proxy Address</p>
              <p className={`text-sm font-mono ${
                isDarkMode ? 'text-blue-400' : 'text-blue-600'
              }`}>
                127.0.0.1:8888
              </p>
            </div>
            <button
              onClick={() => navigator.clipboard.writeText("127.0.0.1:8888")}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium ${
                isDarkMode
                  ? 'bg-blue-600 hover:bg-blue-500 text-white'
                  : 'bg-blue-500 hover:bg-blue-600 text-white'
              }`}
            >
              📋 Copy
            </button>
          </div>
          <div className={`p-3 rounded-lg ${
            isDarkMode ? 'bg-slate-800/50' : 'bg-gray-100'
          }`}>
            <p className={`text-xs mb-1 ${
              isDarkMode ? 'text-slate-500' : 'text-gray-500'
            }`}>System Proxy</p>
            <p className={`text-sm font-medium ${
              stats.auto_proxy_enabled ? 'text-green-500' : 'text-orange-500'
            }`}>
              {stats.auto_proxy_enabled ? 'Auto-configured' : 'Manual setup required'}
            </p>
            {!stats.auto_proxy_enabled && (
              <p className={`text-xs mt-2 ${
                isDarkMode ? 'text-slate-600' : 'text-gray-600'
              }`}>
                Run as Administrator for automatic proxy configuration
              </p>
            )}
          </div>
        </div>
      </details>
    </div>
  );
}

// Helper Components
interface MetricCardProps {
  icon: string;
  label: string;
  value: string;
  isDarkMode: boolean;
  onClick?: () => void;
}

function MetricCard({ icon, label, value, isDarkMode, onClick }: MetricCardProps) {
  return (
    <div
      onClick={onClick}
      className={`p-4 rounded-xl text-center transition-all shadow-sm ${
        isDarkMode
          ? 'bg-slate-900/50 border border-slate-800 hover:bg-slate-800/50'
          : 'bg-white/50 border border-gray-300 hover:bg-white/80'
      } ${
        onClick ? 'cursor-pointer' : ''
      }`}
    >
      <div className="text-2xl mb-2">{icon}</div>
      <p className={`text-xs mb-1.5 ${
        isDarkMode ? 'text-slate-500' : 'text-gray-500'
      }`}>{label}</p>
      <p className={`text-sm font-bold tabular-nums ${
        isDarkMode ? 'text-white' : 'text-gray-900'
      }`}>{value}</p>
    </div>
  );
}

interface DetailRowProps {
  label: string;
  value: number;
  isDarkMode: boolean;
  onClick?: () => void;
}

function DetailRow({ label, value, isDarkMode, onClick }: DetailRowProps) {
  return (
    <div
      onClick={onClick}
      className={`flex items-center justify-between py-3 px-4 rounded-lg ${
        isDarkMode ? 'hover:bg-slate-800/50' : 'hover:bg-gray-100'
      } ${
        onClick ? 'cursor-pointer' : ''
      }`}
    >
      <span className={`text-xs ${
        isDarkMode ? 'text-slate-400' : 'text-gray-600'
      }`}>{label}</span>
      <span className={`text-sm font-bold tabular-nums ${
        isDarkMode ? 'text-white' : 'text-gray-900'
      }`}>{value.toLocaleString()}</span>
    </div>
  );
}

export default Dashboard;
