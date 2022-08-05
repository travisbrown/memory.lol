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
  container!.replaceChildren(container!.children[0]);
  container!.setAttribute("style", "display: none;");

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
  const span = document.createElement("span");
  span.textContent = "Previously: ";
  container.appendChild(span);

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
