chrome.runtime.onMessage.addListener(
  function (request, sender, sendResponse) {
    fetch(
      'https://memory.lol/tw/id/' + request.id,
      {
        method: 'get',
        headers: { 'Content-Type': 'application/json' }
      }
    )
      .then(response => response.json())
      .then(data => sendResponse({ result: data['screen-names'] }));

    return true;
  }
);
