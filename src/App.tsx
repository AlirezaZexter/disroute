import { AnimatePresence, LayoutGroup, MotionConfig, motion } from "motion/react";
import { transitions, panelVariants, viewVariants } from "./motion";
import { AppFooter, TopBar, UpdateNotice, ViewSwitch, WorkspaceHeading } from "./components/AppChrome";
import { ConnectionStatus } from "./components/ConnectionStatus";
import { PersonalConnectionPanel } from "./components/PersonalConnectionPanel";
import { CommunityConnectionPanel } from "./components/CommunityConnectionPanel";
import { ConnectionSidebar } from "./components/ConnectionSidebar";
import { GuideView } from "./components/GuideView";
import { useDisRouteController } from "./useDisRouteController";

function App() {
  const controller = useDisRouteController();

  return (
    <MotionConfig reducedMotion={import.meta.env.MODE === "test" ? "always" : "user"} transition={transitions.normal}>
      <LayoutGroup>
        <main className="shell">
          <TopBar updatePhase={controller.updatePhase} onCheckUpdates={controller.handleCheckUpdates} onHideToTray={controller.hideToTray} />

          <AnimatePresence initial={false}>
            {controller.updatePhase !== "idle" && controller.updatePhase !== "checking" && (
              <UpdateNotice
                phase={controller.updatePhase}
                info={controller.updateInfo}
                progress={controller.updateProgress}
                error={controller.updateError}
                onInstall={controller.handleInstallUpdate}
                onClose={() => controller.setUpdatePhase("idle")}
              />
            )}
          </AnimatePresence>

          <WorkspaceHeading />
          <ViewSwitch view={controller.view} onChange={controller.setView} />
          <ConnectionStatus
            appStatus={controller.appStatus}
            mode={controller.connectionMode}
            busy={controller.busy}
            discordBusy={controller.discordBusy}
            communityBusy={controller.communityBusy}
            connectDisabled={controller.connectDisabled}
            inGuide={controller.view === "guide"}
            onOpenConnection={() => controller.setView("connection")}
            onRestartDiscord={controller.handleRestartDiscord}
            onDisconnect={controller.handleDisconnect}
          />

          <AnimatePresence initial={false} mode="popLayout" custom={controller.view === "connection" ? 1 : -1}>
            {controller.view === "connection" ? (
              <motion.div key="connection" className="content-grid" custom={1} variants={viewVariants} initial="initial" animate="enter" exit="exit">
                <div className="mode-column">
                  <nav className="mode-switch" aria-label="روش اتصال">
                    {([ ["personal", "کانفیگ شخصی"], ["community", "اتصال سریع رایگان"] ] as const).map(([id, label]) => (
                      <button type="button" key={id} aria-pressed={controller.connectionMode === id} onClick={() => controller.setConnectionMode(id)}>
                        {controller.connectionMode === id && <motion.span className="selected-mode" layoutId="selected-mode" transition={transitions.layout} />}
                        <span>{label}</span>
                      </button>
                    ))}
                  </nav>

                  <AnimatePresence initial={false} mode="popLayout" custom={controller.connectionMode === "personal" ? -1 : 1}>
                    {controller.connectionMode === "personal" ? (
                      <motion.div key="personal" className="mode-panel" custom={-1} variants={panelVariants} initial="initial" animate="enter" exit="exit">
                        <PersonalConnectionPanel
                          profile={controller.profile}
                          loading={controller.loading}
                          busy={controller.busy}
                          connected={controller.connected}
                          remember={controller.remember}
                          saved={controller.saved}
                          notice={controller.notice}
                          showSecret={controller.showSecret}
                          configInvalid={controller.configInvalid}
                          hasConfig={controller.hasConfig}
                          unsupportedConfig={controller.unsupportedConfig}
                          detectedProtocol={controller.detectedProtocol}
                          confirmForget={controller.confirmForget}
                          onSubmit={controller.handleSubmit}
                          onProfileChange={controller.updateProfile}
                          onConfigBlur={() => controller.setConfigTouched(true)}
                          onShowSecretChange={controller.setShowSecret}
                          onRememberChange={controller.handleRememberChange}
                          onSave={controller.handleSave}
                          onConfirmForgetChange={controller.setConfirmForget}
                          onForget={controller.handleForget}
                        />
                      </motion.div>
                    ) : (
                      <motion.div key="community" className="mode-panel" custom={1} variants={panelVariants} initial="initial" animate="enter" exit="exit">
                        <CommunityConnectionPanel
                          community={controller.community}
                          results={controller.communityResults}
                          busy={controller.communityBusy}
                          error={controller.communityError}
                          warningChecked={controller.warningChecked}
                          sourceDraft={controller.sourceDraft}
                          connected={controller.connected}
                          onSubmit={controller.handleCommunityConnect}
                          onWarningChecked={controller.setWarningChecked}
                          onAcknowledge={controller.acknowledgeCommunity}
                          onRefresh={controller.handleCommunityRefresh}
                          onScan={controller.handleCommunityScan}
                          onCancelScan={controller.cancelCommunityScan}
                          onFailoverChange={controller.updateFailover}
                          onReplaceSources={controller.replaceSources}
                          onSourceDraftChange={controller.setSourceDraft}
                          onAddSource={controller.addSource}
                          onClearData={controller.handleClearCommunityData}
                        />
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
                <ConnectionSidebar appStatus={controller.appStatus} />
              </motion.div>
            ) : (
              <motion.div key="guide" custom={-1} variants={viewVariants} initial="initial" animate="enter" exit="exit" className="guide-view-wrap">
                <GuideView />
              </motion.div>
            )}
          </AnimatePresence>
          <AppFooter />
        </main>
      </LayoutGroup>
    </MotionConfig>
  );
}

export default App;
