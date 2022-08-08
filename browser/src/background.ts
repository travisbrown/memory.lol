chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.id) {
    fetch(`https://api.memory.lol/v1/tw/id/${request.id}`, {
      method: "get",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
    })
      .then((response) => response.json())
      .then((data) => sendResponse({ result: data.screen_names }));
  } else if (request.screenName) {
    fetch(`https://api.memory.lol/v1/tw/${request.screenName}`, {
      method: "get",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
    })
      .then((response) => response.json())
      .then((data) => sendResponse({ result: data.accounts }));
  }

  return true;
});
