import { useState, useEffect } from "react";
import Logs from "./components/Logs";

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
  exit_country: string | null;
}



function App() {
  const [stats, setStats] = useState<Stats>({
    tor_connected: false,
    kill_switch_active: false,
    requests_blocked: 0,
    trackers_blocked: 0,
    webrtc_blocked: 0,
    ipv6_blocked: 0,
    total_requests: 0,
    proxy_running: false,
    auto_proxy_enabled: false,
    uptime_seconds: 0,
    security_threats_detected: 0,
    exit_country: null,
  });

  const [logs, setLogs] = useState<any[]>([]);
  const [activeTab, setActiveTab] = useState<'dashboard' | 'logs'>('dashboard');
  const [isConnecting, setIsConnecting] = useState(false);
  const [showDonateModal, setShowDonateModal] = useState(false);
  const [copiedAddress, setCopiedAddress] = useState<string | null>(null);
  const [logFilter, setLogFilter] = useState<string>('all');

  const cryptoAddresses = {
    btc: "3CpHZqjxvQvXz64drZZxA6m3NWzKF1LCHX",
    eth: "0x3b867d70DD7C087374D05910F69C9BD2685b074D",
  };

  const copyToClipboard = (address: string, crypto: string) => {
    navigator.clipboard.writeText(address);
    setCopiedAddress(crypto);
    setTimeout(() => setCopiedAddress(null), 2000);
  };

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch("http://127.0.0.1:3030/api/stats");
        const data = await response.json();
        setStats(data);
      } catch (error) {
        console.error("Failed to fetch stats:", error);
      }
    };

    const fetchLogs = async () => {
      try {
        const response = await fetch("http://127.0.0.1:3030/api/logs");
        const data = await response.json();
        setLogs(data || []);
      } catch (error) {
        console.error("Failed to fetch logs:", error);
      }
    };

    fetchStats();
    fetchLogs();
    const interval = setInterval(() => {
      fetchStats();
      fetchLogs();
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const toggleConnection = async () => {
    setIsConnecting(true);
    try {
      const response = await fetch("http://127.0.0.1:3030/api/connection", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          connect: !stats.tor_connected,
        }),
      });
      const data = await response.json();
      setStats((prev) => ({ ...prev, tor_connected: data.is_connected || data.tor_connected }));
    } catch (error) {
      console.error("Failed to toggle connection:", error);
    } finally {
      setIsConnecting(false);
    }
  };



  const toggleKillSwitch = async () => {
    try {
      const response = await fetch("http://127.0.0.1:3030/api/kill-switch", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ enabled: !stats.kill_switch_active }),
      });
      const data = await response.json();
      setStats((prev) => ({ ...prev, kill_switch_active: data.enabled }));
    } catch (error) {
      console.error("Failed to toggle kill switch:", error);
    }
  };

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };



  return (
    <div className="min-h-screen bg-gradient-to-br from-[#1a1a2e] via-[#16213e] to-[#0f1624] text-white flex flex-col relative overflow-hidden">
      {/* Subtle Grid Pattern Background */}
      <div 
        className="absolute inset-0 opacity-[0.03] pointer-events-none"
        style={{
          backgroundImage: 'radial-gradient(circle at 2px 2px, rgba(255, 255, 255, 0.15) 1px, transparent 0)',
          backgroundSize: '40px 40px'
        }}
      ></div>
      <div className="container mx-auto px-6 py-6 max-w-lg flex flex-col flex-1 relative z-10">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-3">
            <svg className="w-8 h-8 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
            <h1 className="text-lg font-bold text-white">DUL Privacy</h1>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setShowDonateModal(true)}
              className="px-3 py-2 bg-gradient-to-r from-purple-500/10 to-pink-500/10 hover:from-purple-500/20 hover:to-pink-500/20 border border-purple-500/30 rounded-xl transition-all flex items-center space-x-2"
            >
              <svg className="w-5 h-5 text-purple-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span className="text-sm font-medium text-purple-300">Donate</span>
            </button>
            <button 
            onClick={() => setActiveTab(activeTab === 'dashboard' ? 'logs' : 'dashboard')}
            className="p-2.5 bg-white/5 hover:bg-white/10 rounded-xl transition-all"
          >
            {activeTab === 'dashboard' ? (
              <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            ) : (
              <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
            )}
          </button>
          </div>
        </div>

        {activeTab === 'dashboard' ? (
          <div className="flex flex-col flex-1">
            {/* Top Section - Status and Timer */}
            <div className="space-y-4 mb-6">
              {/* Status Badge */}
              <div className="flex justify-center">
                <div className={`inline-flex items-center space-x-2 px-5 py-2.5 rounded-full backdrop-blur-xl ${
                  stats.tor_connected 
                    ? 'bg-gradient-to-r from-emerald-500/20 to-teal-500/20 border border-emerald-400/40' 
                    : 'bg-white/5 border border-white/10'
                }`}>
                  <div className={`w-2 h-2 rounded-full ${stats.tor_connected ? 'bg-emerald-400 shadow-lg shadow-emerald-400/50' : 'bg-gray-500'}`}></div>
                  <span className="text-sm font-semibold tracking-wide">
                    {stats.tor_connected ? 'PROTECTED' : 'DISCONNECTED'}
                  </span>
                </div>
              </div>

              {/* Timer Display */}
              <div className="text-center">
                <div className="text-5xl font-light tracking-widest mb-1 text-purple-100">
                  {formatUptime(stats.uptime_seconds)}
                </div>
                <div className="text-xs text-purple-300/60 uppercase tracking-wider">Connection Time</div>
              </div>
            </div>

            {/* Middle Section - Power Button */}
            <div className="flex justify-center items-center flex-1">
              <button
                onClick={toggleConnection}
                disabled={isConnecting}
                className={`relative group w-48 h-48 rounded-full transition-all duration-700 transform ${
                  isConnecting ? 'scale-95 opacity-75' : 'hover:scale-105'
                } ${
                  stats.tor_connected
                    ? 'bg-gradient-to-br from-emerald-400 via-teal-500 to-cyan-600 shadow-2xl shadow-emerald-500/40'
                    : 'bg-gradient-to-br from-purple-500 via-violet-600 to-indigo-700 shadow-2xl shadow-purple-500/40'
                }`}
              >
                <div className="absolute inset-2 rounded-full bg-[#1a1a2e]/90 backdrop-blur-md flex items-center justify-center border-2 border-white/10">
                  {isConnecting ? (
                    <div className="w-16 h-16 border-4 border-purple-300/30 border-t-purple-300 rounded-full animate-spin"></div>
                  ) : (
                    <svg
                      className="w-24 h-24 text-white drop-shadow-2xl"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={1.5}
                        d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z"
                      />
                    </svg>
                  )}
                </div>
              </button>
            </div>

            {/* Bottom Section - Stats and System Status */}
            <div className="space-y-3 px-6 pb-4">
              {/* Stats List */}
              <div className="space-y-2">
                <button 
                  onClick={() => { setLogFilter('tracker'); setActiveTab('logs'); }}
                  className="w-full px-4 py-3 bg-gradient-to-r from-purple-500/15 to-pink-500/15 rounded-xl backdrop-blur-xl border border-purple-500/30 hover:border-purple-400/50 hover:shadow-lg hover:shadow-purple-500/20 transition-all cursor-pointer flex items-center justify-between group"
                >
                  <div className="flex items-center space-x-3">
                    <svg className="w-5 h-5 text-purple-400 group-hover:text-purple-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                    </svg>
                    <span className="text-sm font-medium text-purple-300 uppercase tracking-wide">Trackers</span>
                  </div>
                  <span className="text-xl font-bold text-purple-300">{stats.trackers_blocked.toLocaleString()}</span>
                </button>

                <button 
                  onClick={() => { setLogFilter('all'); setActiveTab('logs'); }}
                  className="w-full px-4 py-3 bg-gradient-to-r from-blue-500/15 to-cyan-500/15 rounded-xl backdrop-blur-xl border border-blue-500/30 hover:border-blue-400/50 hover:shadow-lg hover:shadow-blue-500/20 transition-all cursor-pointer flex items-center justify-between group"
                >
                  <div className="flex items-center space-x-3">
                    <svg className="w-5 h-5 text-blue-400 group-hover:text-blue-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z" />
                    </svg>
                    <span className="text-sm font-medium text-blue-300 uppercase tracking-wide">Requests</span>
                  </div>
                  <span className="text-xl font-bold text-blue-300">{stats.requests_blocked.toLocaleString()}</span>
                </button>

                <button 
                  onClick={() => { setLogFilter('webrtc'); setActiveTab('logs'); }}
                  className="w-full px-4 py-3 bg-gradient-to-r from-violet-500/15 to-purple-500/15 rounded-xl backdrop-blur-xl border border-violet-500/30 hover:border-violet-400/50 hover:shadow-lg hover:shadow-violet-500/20 transition-all cursor-pointer flex items-center justify-between group"
                >
                  <div className="flex items-center space-x-3">
                    <svg className="w-5 h-5 text-violet-400 group-hover:text-violet-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                    <span className="text-sm font-medium text-violet-300 uppercase tracking-wide">WebRTC</span>
                  </div>
                  <span className="text-xl font-bold text-violet-300">{stats.webrtc_blocked.toLocaleString()}</span>
                </button>

                <button 
                  onClick={() => { setLogFilter('ipv6'); setActiveTab('logs'); }}
                  className="w-full px-4 py-3 bg-gradient-to-r from-indigo-500/15 to-blue-500/15 rounded-xl backdrop-blur-xl border border-indigo-500/30 hover:border-indigo-400/50 hover:shadow-lg hover:shadow-indigo-500/20 transition-all cursor-pointer flex items-center justify-between group"
                >
                  <div className="flex items-center space-x-3">
                    <svg className="w-5 h-5 text-indigo-400 group-hover:text-indigo-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                    </svg>
                    <span className="text-sm font-medium text-indigo-300 uppercase tracking-wide">IPv6</span>
                  </div>
                  <span className="text-xl font-bold text-indigo-300">{stats.ipv6_blocked.toLocaleString()}</span>
                </button>

                <button 
                  onClick={() => { setLogFilter('everything'); setActiveTab('logs'); }}
                  className="w-full px-4 py-3 bg-gradient-to-r from-emerald-500/15 to-teal-500/15 rounded-xl backdrop-blur-xl border border-emerald-500/30 hover:border-emerald-400/50 hover:shadow-lg hover:shadow-emerald-500/20 transition-all cursor-pointer flex items-center justify-between group"
                >
                  <div className="flex items-center space-x-3">
                    <svg className="w-5 h-5 text-emerald-400 group-hover:text-emerald-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                    </svg>
                    <span className="text-sm font-medium text-emerald-300 uppercase tracking-wide">Total</span>
                  </div>
                  <span className="text-xl font-bold text-emerald-300">{stats.total_requests.toLocaleString()}</span>
                </button>

                <button 
                  onClick={() => { setLogFilter('security'); setActiveTab('logs'); }}
                  className="w-full px-4 py-3 bg-gradient-to-r from-orange-500/15 to-red-500/15 rounded-xl backdrop-blur-xl border border-orange-500/30 hover:border-orange-400/50 hover:shadow-lg hover:shadow-orange-500/20 transition-all cursor-pointer flex items-center justify-between group"
                >
                  <div className="flex items-center space-x-3">
                    <svg className="w-5 h-5 text-orange-400 group-hover:text-orange-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                    <span className="text-sm font-medium text-orange-300 uppercase tracking-wide">Threats</span>
                  </div>
                  <span className="text-xl font-bold text-orange-300">{stats.security_threats_detected.toLocaleString()}</span>
                </button>
              </div>

            {/* Kill Switch */}
            <div className="flex items-center justify-between px-6 py-4 bg-gradient-to-r from-red-500/15 to-orange-500/15 rounded-2xl backdrop-blur-xl border border-red-500/30 shadow-lg shadow-red-500/10">
              <div className="flex items-center space-x-3">
                <svg className="w-6 h-6 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                <div>
                  <div className="text-sm font-semibold text-red-300">Kill Switch</div>
                  <div className="text-xs text-red-400/60">Auto-disconnect protection</div>
                </div>
              </div>
              <button
                onClick={toggleKillSwitch}
                className={`relative w-14 h-7 rounded-full transition-all duration-300 ${
                  stats.kill_switch_active ? 'bg-gradient-to-r from-red-500 to-orange-600' : 'bg-white/10'
                }`}
              >
                <div
                  className={`absolute top-0.5 left-0.5 w-6 h-6 bg-white rounded-full shadow-lg transition-transform duration-300 ${
                    stats.kill_switch_active ? 'translate-x-7' : 'translate-x-0'
                  }`}
                ></div>
              </button>
            </div>

            {/* System Status */}
            <div className="px-6 py-4 bg-white/10 rounded-2xl backdrop-blur-xl border border-white/20 shadow-lg">
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center space-x-2">
                  <div className={`w-2 h-2 rounded-full ${
                    stats.proxy_running ? 'bg-emerald-400 shadow-lg shadow-emerald-400/50' : 'bg-gray-600'
                  }`}></div>
                  <span className="text-xs text-gray-300">Proxy</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className={`w-2 h-2 rounded-full ${
                    stats.tor_connected ? 'bg-emerald-400 shadow-lg shadow-emerald-400/50' : 'bg-gray-600'
                  }`}></div>
                  <span className="text-xs text-gray-300">Tor Network</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className={`w-2 h-2 rounded-full ${
                    stats.auto_proxy_enabled ? 'bg-emerald-400 shadow-lg shadow-emerald-400/50' : 'bg-gray-600'
                  }`}></div>
                  <span className="text-xs text-gray-300">Auto Proxy</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className={`w-2 h-2 rounded-full ${
                    stats.kill_switch_active ? 'bg-orange-400 shadow-lg shadow-orange-400/50' : 'bg-gray-600'
                  }`}></div>
                  <span className="text-xs text-gray-300">Kill Switch</span>
                </div>
              </div>
            </div>
            </div>
          </div>
        ) : (
          <Logs logs={logs} isDarkMode={true} initialFilter={logFilter} />
        )}

        {/* Crypto Donation Modal */}
        {showDonateModal && (
          <div className="fixed inset-0 bg-black/70 backdrop-blur-md flex items-center justify-center z-50 p-4" onClick={() => setShowDonateModal(false)}>
            <div className="bg-gradient-to-br from-[#1a1a2e] via-[#16213e] to-[#0f1624] rounded-3xl p-8 max-w-md w-full border border-purple-500/20 shadow-2xl" onClick={(e) => e.stopPropagation()}>
              <div className="flex items-center justify-between mb-6">
                <div className="flex items-center space-x-3">
                  <svg className="w-8 h-8 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                  </svg>
                  <h2 className="text-2xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">Support DUL Privacy</h2>
                </div>
                <button
                  onClick={() => setShowDonateModal(false)}
                  className="p-2 hover:bg-white/10 rounded-lg transition-all"
                >
                  <svg className="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>

              <p className="text-gray-300 text-sm mb-6">
                Help us keep DUL Privacy Suite free and open-source. Your support enables continued development and service maintenance.
              </p>

              <div className="space-y-3">
                {/* Bitcoin */}
                <div className="p-4 bg-white/5 rounded-xl border border-purple-500/20 backdrop-blur-xl hover:border-purple-500/40 transition-colors">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      <svg className="w-5 h-5 text-orange-400" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M23.638 14.904c-1.602 6.43-8.113 10.34-14.542 8.736C2.67 22.05-1.244 15.525.362 9.105 1.962 2.67 8.475-1.243 14.9.358c6.43 1.605 10.342 8.115 8.738 14.548v-.002zm-6.35-4.613c.24-1.59-.974-2.45-2.64-3.03l.54-2.153-1.315-.33-.525 2.107c-.345-.087-.705-.167-1.064-.25l.526-2.127-1.32-.33-.54 2.165c-.285-.067-.565-.132-.84-.2l-1.815-.45-.35 1.407s.975.225.955.236c.535.136.63.486.615.766l-1.477 5.92c-.075.166-.24.406-.614.314.015.02-.96-.24-.96-.24l-.66 1.51 1.71.426.93.242-.54 2.19 1.32.327.54-2.17c.36.1.705.19 1.05.273l-.51 2.154 1.32.33.545-2.19c2.24.427 3.93.257 4.64-1.774.57-1.637-.03-2.58-1.217-3.196.854-.193 1.5-.76 1.68-1.93h.01zm-3.01 4.22c-.404 1.64-3.157.75-4.05.53l.72-2.9c.896.23 3.757.67 3.33 2.37zm.41-4.24c-.37 1.49-2.662.735-3.405.55l.654-2.64c.744.18 3.137.524 2.75 2.084v.006z"/>
                      </svg>
                      <span className="font-semibold text-orange-300">Bitcoin</span>
                    </div>
                    <button
                      onClick={() => copyToClipboard(cryptoAddresses.btc, 'btc')}
                      className="px-3 py-1 bg-purple-500/20 hover:bg-purple-500/30 rounded-lg text-xs font-medium text-purple-300 transition-all"
                    >
                      {copiedAddress === 'btc' ? '✓ Copied!' : 'Copy'}
                    </button>
                  </div>
                  <div className="text-xs text-gray-400 break-all font-mono bg-black/30 p-2 rounded">
                    {cryptoAddresses.btc}
                  </div>
                </div>

                {/* Ethereum */}
                <div className="p-4 bg-white/5 rounded-xl border border-purple-500/20 backdrop-blur-xl hover:border-purple-500/40 transition-colors">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      <svg className="w-5 h-5 text-blue-400" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M11.944 17.97L4.58 13.62 11.943 24l7.37-10.38-7.372 4.35h.003zM12.056 0L4.69 12.223l7.365 4.354 7.365-4.35L12.056 0z"/>
                      </svg>
                      <span className="font-semibold text-blue-300">Ethereum</span>
                    </div>
                    <button
                      onClick={() => copyToClipboard(cryptoAddresses.eth, 'eth')}
                      className="px-3 py-1 bg-purple-500/20 hover:bg-purple-500/30 rounded-lg text-xs font-medium text-purple-300 transition-all"
                    >
                      {copiedAddress === 'eth' ? '✓ Copied!' : 'Copy'}
                    </button>
                  </div>
                  <div className="text-xs text-gray-400 break-all font-mono bg-black/30 p-2 rounded">
                    {cryptoAddresses.eth}
                  </div>
                </div>
              </div>

              <div className="mt-6 p-3 bg-purple-500/10 border border-purple-500/20 rounded-xl">
                <p className="text-xs text-purple-300 text-center">
                  Thank you for supporting open-source privacy tools!
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
