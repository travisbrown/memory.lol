let started = false;
let currentUrl = null;
let current_screen_name = null;

const observe = () => {
	const observer = new MutationObserver((mutations) => {
		if (mutations.length) {
			if (window.location.href !== currentUrl) {
				started = false;
				currentUrl = window.location.href;
				current_screen_name = get_current_screen_name();
			}

			if (!started && current_screen_name !== null) {
				const current = document.getElementById("memory-lol");
				if (current !== null) {
					current.remove();
				}

				const followButton = document.querySelector("div[data-testid$='-follow']");

				if (followButton !== null && !started) {
					started = true;
					const testid = followButton.getAttribute("data-testid");
					const id = testid.substring(0, testid.length - 7);

					chrome.runtime.sendMessage({ id: id }, function (response) {
						let screen_names = response.result;
						let filtered = [];

						for (var screen_name in screen_names) {
							if (screen_name.toLowerCase() !== current_screen_name.toLowerCase()) {
								filtered.push([screen_name, screen_names[screen_name]]);
							}
						}

						if (filtered.length > 0) {
							const userName = document.querySelector("div[data-testid='UserName']");
							const userDescription = document.querySelector("div[data-testid='UserDescription']");
							const userJoinDate = document.querySelector("span[data-testid='UserJoinDate']");
							const UserProfileHeader_Items = document.querySelector("div[data-testid='UserProfileHeader_Items']");
							const userUrl = document.querySelector("a[data-testid='UserUrl']");

							if (userName) {
								let div = document.createElement("div");
								div.setAttribute("id", "memory-lol");
								if (userDescription) {
									div.setAttribute("class", `${userDescription.getAttribute("class")} memory-lol`);
								} else {
									div.setAttribute("class", `${userJoinDate.getAttribute("class")} memory-lol`);
								}

								let span = document.createElement("span");
								if (userDescription) {
									span.setAttribute("class", userDescription.firstElementChild.getAttribute("class"));
								} else {
									span.setAttribute("class", `${userJoinDate.children[1].getAttribute("class")} ${UserProfileHeader_Items.getAttribute("class")}`);
								}
								span.innerHTML = "Previously: ";
								div.appendChild(span);

								for (var i = 0; i < filtered.length; i += 1) {
									let pair = filtered[i];
									let link = document.createElement("a");
									if (userUrl) {
										link.setAttribute("class", userUrl.getAttribute("class"));
									} else if (userDescription) {
										link.setAttribute("class", userDescription.getAttribute("class"));
									} else {
										span.setAttribute("class", `${userJoinDate.children[1].getAttribute("class")} ${UserProfileHeader_Items.getAttribute("class")}`);
									}

									link.setAttribute("href", `http://web.archive.org/web/*/https://twitter.com/${pair[0]}/status/*`);
									link.innerHTML = `@${pair[0]}`;

									if (pair[1] !== null) {
										if (pair[1].length == 1) {
											link.setAttribute("title", pair[1][0]);
										} else if (pair[1].length == 2) {
											link.setAttribute("title", `${pair[1][0]} to ${pair[1][1]}`);
										}
									}

									div.appendChild(link);

									if (i < filtered.length - 1) {
										let span = document.createElement("span");
										if (userDescription) {
											span.setAttribute("class", userDescription.firstElementChild.getAttribute("class"));
										} else {
											span.setAttribute("class", `${userJoinDate.children[1].getAttribute("class")} ${UserProfileHeader_Items.getAttribute("class")}`);
										}
										span.innerHTML = " | ";
										div.appendChild(span);
									}
								}

								userName.parentNode.insertBefore(div, userName.nextSibling);

							}
						}
					});
				}
			}
		}
	});

	observer.observe(document, {
		childList: true,
		subtree: true,
	});
};

function get_current_screen_name() {
	const screen_name_path_re = /^\/(\w+)$/;
	const path = window.location.pathname;
	const screen_name_match = path.match(screen_name_path_re);
	if (screen_name_match !== null) {
		const screen_name = screen_name_match[1];

		if (screen_name !== "home") {
			return screen_name;
		}
	}
	return null;
}

const init = () => {
	observe();
}

init();
