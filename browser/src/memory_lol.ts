let container: Element | null = null;
let containerClasses: string | null = null;
let linkClasses: string | null = null;
let spanClasses: string | null = null;
let currentUrl: string | null = null;

/**
 * Extract Twitter profile information from a script element.
 *
 * @param element A JSON-LD script element
 * @returns The Twitter user ID and screen name of the current profile
 */
const getUserInfo: (element: Element) => [string, string] | null = (
  element
) => {
  if (element) {
    const ldJson: { author: { additionalName: string; identifier: string } } =
      JSON.parse(element.textContent!);
    const author = ldJson.author;

    return [author.identifier, author.additionalName];
  }

  return null;
};

const updatePastScreenNames: (id: string, screenName: string) => void = (
  id,
  screenName
) => {
  container!.setAttribute("style", "display: none;");
  container!.replaceChildren();

  chrome.runtime.sendMessage({ id: id }, function (response) {
    const currentScreenName = screenName.toLowerCase();
    const screenNames = response.result;
    const results = [];

    for (const pastScreenName in screenNames) {
      if (pastScreenName.toLowerCase() !== currentScreenName) {
        results.push([pastScreenName, screenNames[pastScreenName]]);
      }
    }

    if (results.length > 0) {
      container!.removeAttribute("style");
      const span = document.createElement("span");
      span.textContent = "Previously: ";
      container!.appendChild(span);
    }

    for (const [index, result] of results.entries()) {
      const [screenName, dates] = result;
      const link = document.createElement("a");
      link.setAttribute("class", linkClasses!);

      link.setAttribute(
        "href",
        `http://web.archive.org/web/*/https://twitter.com/${screenName}/status/*`
      );
      link.textContent = `@${screenName}`;

      if (dates) {
        if (dates.length == 1) {
          link.setAttribute("title", dates[0]);
        } else if (dates.length == 2) {
          link.setAttribute("title", `${dates[0]} to ${dates[1]}`);
        }
      }

      container!.appendChild(link);

      if (index < results.length - 1) {
        const span = document.createElement("span");
        span.setAttribute("class", spanClasses!);
        span.textContent = " | ";
        container!.appendChild(span);
      }
    }
  });
};

const updateForNonExistent: (screenName: string) => void = (screenName) => {
  container!.replaceChildren();
  container!.setAttribute("style", "display: none;");

  chrome.runtime.sendMessage(
    { screenName: screenName },
    function (response: { result: [{ id_str: string; screen_names: any }] }) {
      const currentScreenName = screenName.toLowerCase();
      const accounts = response.result;
      const possibleIds = [];
      const possibleScreenNames = [];

      for (const account of accounts) {
        if (account.id_str) {
          possibleIds.push(account.id_str);
        }

        for (const pastScreenName in account.screen_names) {
          if (pastScreenName.toLowerCase() !== currentScreenName) {
            possibleScreenNames.push([
              pastScreenName,
              account.screen_names[pastScreenName],
            ]);
          }
        }
      }

      if (accounts.length > 0) {
        container!.removeAttribute("style");

        const div = document.createElement("div");
        div.setAttribute("id", "non-existent-user-ids");

        const span = document.createElement("span");
        span.textContent =
          possibleIds.length > 1 ? "Possible IDs: " : "Possible ID: ";
        div.appendChild(span);

        for (const [index, id] of possibleIds.entries()) {
          const link = document.createElement("a");
          link.setAttribute("class", linkClasses!);

          link.setAttribute(
            "href",
            `https://twitter.com/intent/user?user_id=${id}`
          );
          link.textContent = id;

          div.appendChild(link);

          if (index < possibleIds.length - 1) {
            const span = document.createElement("span");
            span.setAttribute("class", spanClasses!);
            span.textContent = " | ";
            div.appendChild(span);
          }
        }

        container!.appendChild(div);

        if (possibleScreenNames.length > 0) {
          const div = document.createElement("div");
          div.setAttribute("id", "non-existent-user-screen-names");

          const span = document.createElement("span");
          span.textContent =
            possibleScreenNames.length > 1
              ? "Possible previous screen names: "
              : "Possible previous screen name: ";
          div.appendChild(span);

          for (const [index, result] of possibleScreenNames.entries()) {
            const [screenName, dates] = result;
            const link = document.createElement("a");
            link.setAttribute("class", linkClasses!);

            link.setAttribute(
              "href",
              `http://web.archive.org/web/*/https://twitter.com/${screenName}/status/*`
            );
            link.textContent = `@${screenName}`;

            if (dates) {
              if (dates.length == 1) {
                link.setAttribute("title", dates[0]);
              } else if (dates.length == 2) {
                link.setAttribute("title", `${dates[0]} to ${dates[1]}`);
              }
            }

            div.appendChild(link);

            if (index < possibleScreenNames.length - 1) {
              const span = document.createElement("span");
              span.setAttribute("class", spanClasses!);
              span.textContent = " | ";
              div.appendChild(span);
            }
          }

          container!.appendChild(div);
        }
      }
    }
  );
};

const observer = new MutationObserver((mutations) => {
  for (const mutation of mutations) {
    if (mutation.type === "childList") {
      for (const node of mutation.removedNodes) {
        if (node.nodeName === "SCRIPT") {
          const element = node as HTMLElement;

          if (
            element.getAttribute("type") === "application/ld+json" &&
            // Firefox seems to remove and re-add the script element?
            currentUrl != window.location.toString()
          ) {
            container!.replaceChildren(container!.children[0]);
            container!.setAttribute("style", "display: none;");
          }
        }
      }

      for (const node of mutation.addedNodes) {
        if (node.nodeType == Node.ELEMENT_NODE) {
          const element = node as Element;

          if (containerClasses === null) {
            const linkTemplate = element.querySelector(
              "a[href='/i/keyboard_shortcuts']"
            );

            if (linkTemplate) {
              if (linkTemplate.hasAttribute("class")) {
                linkClasses = linkTemplate.getAttribute("class")!;
              }

              if (linkTemplate.previousElementSibling) {
                const spanTemplate =
                  linkTemplate.previousElementSibling.querySelector("span");

                if (spanTemplate && spanTemplate.hasAttribute("class")) {
                  spanClasses = spanTemplate.getAttribute("class");
                }

                if (linkTemplate.previousElementSibling.hasAttribute("class")) {
                  containerClasses =
                    linkTemplate.previousElementSibling.getAttribute("class")!;
                  container!.setAttribute("class", containerClasses);
                }
              }
            }
          }

          const userNameDiv = element.querySelector(
            "div[data-testid='UserName']"
          );

          if (userNameDiv) {
            userNameDiv.parentNode!.insertBefore(
              container!,
              userNameDiv.nextSibling
            );
          }

          // We're on an account profile that is either suspended or non-existent.
          const emptyState = element.querySelector(
            "div[data-testid='emptyState']"
          );

          if (emptyState) {
            const primaryColumn = document.querySelector(
              "div[data-testid='primaryColumn']"
            );
            if (primaryColumn) {
              const screenNameSpan = document.evaluate(
                ".//span[starts-with(text(), '@')]",
                primaryColumn,
                null,
                XPathResult.FIRST_ORDERED_NODE_TYPE,
                null
              ).singleNodeValue;
              if (screenNameSpan) {
                screenNameSpan.parentNode!.insertBefore(
                  container!,
                  screenNameSpan.nextSibling
                );

                let screenName = screenNameSpan.textContent?.substring(1);

                if (screenName) {
                  updateForNonExistent(screenName);
                }
              }
            }
          }

          if (element.tagName === "SCRIPT") {
            if (
              element.getAttribute("type") === "application/ld+json" &&
              // Firefox seems to remove and re-add the script element?
              currentUrl != window.location.toString()
            ) {
              const userInfo = getUserInfo(element);

              if (userInfo) {
                currentUrl = window.location.toString();
                updatePastScreenNames(userInfo[0], userInfo[1]);
              }
            }
          }
        }
      }
    }
  }
});

const init = () => {
  container = document.createElement("div");
  container.setAttribute("id", "memory-lol");
  container.setAttribute("style", "display: none");

  const ldScript = document.querySelector("script[type='application/ld+json']");

  if (ldScript) {
    const userInfo = getUserInfo(ldScript);

    if (userInfo) {
      currentUrl = window.location.toString();
      updatePastScreenNames(userInfo[0], userInfo[1]);
    }
  }

  observer.observe(document, {
    childList: true,
    subtree: true,
  });
};

init();
