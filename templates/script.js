function toggleEvent(event, commandId) {
    event.stopPropagation();
    toggleCommand(commandId);
}

function toggleCommand(commandId, onlyExpand = false) {
    const triangleElement = document.getElementById("triangle-" + commandId);
    if (triangleElement === null) {
        return;
    }
    const isExpanded = triangleElement.classList.contains('expanded');

    if (isExpanded && onlyExpand) {
        return
    }

    const treeItem = triangleElement.parentElement;
    const childrenContainer = treeItem.nextElementSibling;

    triangleElement.classList.toggle('expanded');

    if (isExpanded) {
        triangleElement.classList.remove('expanded');
        childrenContainer.classList.add('hidden');
    } else {
        triangleElement.classList.add('expanded');
        childrenContainer.classList.remove('hidden');
    }
}


function selectCommand(command, element) {
    document.querySelectorAll('.tree-item').forEach(item => {
        item.classList.remove('selected');
    });
    element.classList.add('selected');
    showCommandDetails(command);
    currentCommand = command;
}

function showCommandDetails(command) {
    const data = commandData[command];

    if (!data) {
        return;
    }

    const usages = data.usages.map(usage => {
        return `<div class="command-signature">${usage}</div>`;
    }).join('');

    let desc = "";
    if (data.description) {
        desc = `<div style="margin-top: 8px;">${data.description}</div>`
    }

    document.getElementById('commandInfo').innerHTML = `
                 <div class="info-brief">${data.brief}</div>
                ${usages} ${desc}
            `;

    if (data.arguments.length === 0) {
        document.getElementById('argumentsInfo').innerHTML = '';
    } else {
        let items = data.arguments.map(argument => {
            let desc = "";
            let className = "";
            let showMore = "";
            let onClick = "";
            if (argument.description) {
                desc = `<div class="option-full-doc">${argument.description}</div>`;
                className = "option-item-expandable";
                showMore = '<div class="show-more-indicator">Show more</div>';
                onClick = 'onClick="toggleFullDoc(this)"';
            }
            return `<div class="option-item ${className}" ${onClick}">
                    <div class="option-header">
                        <div class="option-main">
                          <div class="argument-name">${argument.name}</div>
                          <div class="option-description">${argument.brief}</div>
                        </div>
                        ${showMore}
                    </div>
                    ${desc}
                </div>`
        });
        document.getElementById('argumentsInfo').innerHTML = `<div class="info-section">
                    <div class="info-header">
                        Arguments
                    </div>
                    <div class="info-content" id="optionsList">
                        ${items.join("")}
                    </div>
                </div>`;
    }

    if (data.categories.length === 0) {
        document.getElementById('categoryList').innerHTML = '';
    } else {

        document.getElementById('categoryList').innerHTML = data.categories.map(category => {
                const items = category.options.map(option => {
                    let desc = "";
                    let className = "";
                    let showMore = "";
                    let onClick = "";
                    if (option.description) {
                        desc = `<div class="option-full-doc">${option.description}</div>`;
                        className = "option-item-expandable";
                        showMore = '<div class="show-more-indicator">Show more</div>';
                        onClick = 'onClick="toggleFullDoc(this)"';
                    }
                    let short = "";
                    if (option.short) {
                        short = `<span class="option-short">${option.short},</span> `
                    }
                    return `<div class="option-item ${className}" id="${option.id}" ${onClick}">
                                    <div class="option-header">
                                        <div class="option-main">
                                          <div class="option-name">${short}${option.long}</div>
                                          <div class="option-description">${option.brief}</div>
                                        </div>
                                        ${showMore}
                                    </div>
                                    ${desc}
                                </div>`
                });
                return `<div class="info-section">
                            <div class="info-header">
                                ${category.title}
                            </div>
                            <div class="info-content" id="optionsList">
                                ${items.join("")}
                            </div>
                        </div>`
            }
        ).join("");
    }
}

function expandCommandsTo(id) {
    toggleCommand(id, true);
    let command = commandData[id];
    if (command.parent) {
        expandCommandsTo(command.parent);
    }
}

function performSearch() {
    resetHighlights();
    const searchTerm = document.getElementById('searchInput').value.toLowerCase();
    if (!searchTerm) {
        return;
    }

    const results = [];
    const keys = Object.keys(commandData).toSorted();
    keys.forEach(command => {
        const data = commandData[command];
        if (data.name.toLowerCase().includes(searchTerm)) {
            results.push({command: command});
        }
        data.categories.forEach(category => {
            category.options.forEach(option => {
                if (option.long.toLowerCase().includes(searchTerm) || option.brief.toLowerCase().includes(searchTerm)) {
                    results.push({command: command, child_id: option.id});
                }
            })
        })
    })
    searchResults = results;
    searchIndex = 0;
    navigateSearch(0);
    updateSearchWidget();
}

function hideSearchWidget() {
    const widget = document.getElementById('searchWidget');
    widget.classList.remove('visible');
}

function resetHighlights() {
    document.querySelectorAll('.search-highlight-command').forEach(item => {
        item.classList.remove('search-highlight-command');
    })
}

function navigateSearch(direction) {
    if (searchResults.length === 0) {
        return;
    }
    const newIndex = searchIndex + direction;
    if (newIndex >= 0 && newIndex < searchResults.length) {
        resetHighlights();
        searchIndex = newIndex;
        let result = searchResults[newIndex];
        let parent = commandData[result.command].parent;
        if (parent) {
            expandCommandsTo(parent);
        }
        const commandNodeId = `node-${result.command}`;
        selectCommand(result.command, document.getElementById(commandNodeId));
        let elementId;
        if (result.child_id) {
            elementId = result.child_id;
        } else {
            elementId = commandNodeId;
        }
        let element = document.getElementById(elementId);
        element.classList.add("search-highlight-command");
        element.scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
        updateSearchWidget();
    }
}

function updateSearchWidget() {
    const widget = document.getElementById('searchWidget');

    if (searchResults.length > 0) {
        const counter = document.getElementById('searchCounter');
        const prevBtn = document.getElementById('searchPrev');
        const nextBtn = document.getElementById('searchNext');

        widget.classList.add('visible');
        counter.textContent = `${searchIndex + 1}/${searchResults.length}`;

        prevBtn.disabled = searchIndex === 0;
        nextBtn.disabled = searchIndex === searchResults.length - 1;
    } else {
        widget.classList.remove('visible');
    }
}

function resetSearch() {
    if (searchResults) {
        resetHighlights();
        hideSearchWidget();
        searchResults = null;
    }
}

function init() {
    const hash = window.location.hash;
    const parts = hash.slice(1).split('/');
    if (parts.length < 2) {
        showCommandDetails("c0");
    }
    let currentId = "c0";
    for (const part of parts.slice(1)) {
        let current = commandData[currentId];
        let found = false;
        if (current.children) {
            for (const child_id of current.children) {
                let child = commandData[child_id];
                if (child.name === part) {
                    found = true;
                    currentId = child_id;
                    break
                }
            }
        }
        if (!found) {
            break
        }
    }
    expandCommandsTo(currentId);
    selectCommand(currentId, document.getElementById(`node-${currentId}`));
}
