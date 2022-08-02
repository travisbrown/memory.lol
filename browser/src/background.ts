chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  fetch(`https://api.memory.lol/v1/tw/id/${request.id}`, {
    method: "get",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
  })
    .then((response) => response.json())
    .then((data) => sendResponse({ result: data.screen_names }));

  return true;
});
