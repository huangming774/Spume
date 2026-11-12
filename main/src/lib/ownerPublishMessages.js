export function ownerFriendPublishToast(item = {}) {
  if (item.changed === false) {
    return `这条友链已经存在于 ${item.path || "友链数据"}。`;
  }

  const commitSha = item.commitSha ? `（commit ${item.commitSha.slice(0, 7)}）` : "";
  if (item.pullRequestURL) {
    const prNumber = item.pullRequestNumber ? ` #${item.pullRequestNumber}` : "";
    return `友链 Pull Request${prNumber} 已创建${commitSha}：${item.pullRequestURL}。合并后服务器会从 master 自动部署。`;
  }

  return `已写入 ${item.path || "友链数据"}${commitSha}，GitHub Actions 会从 ${item.branch || "master"} 部署。`;
}
