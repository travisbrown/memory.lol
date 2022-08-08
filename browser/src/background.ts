const API_ROOT = "https://api.memory.lol/v1";

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.id) {
    fetch(`${API_ROOT}/tw/id/${request.id}`, {
      method: "get",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
    })
      .then((response) => response.json())
      .then((data) => sendResponse({ result: data.screen_names }));
  } else if (request.screenName) {
    fetch(`${API_ROOT}/tw/${request.screenName}`, {
      method: "get",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
    })
      .then((response) => response.json())
      .then((data) => sendResponse({ result: data.accounts }));
  } else if (request.query && request.query === "status") {
    fetch(`${API_ROOT}/login/status`, {
      method: "get",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
    })
      .then((response) => response.json())
      .then((data) => sendResponse({ result: data }));
  }

  return true;
});
