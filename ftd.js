// all ftd_utils are meant to be pure functions only: they can only depend on the
// input passed, not on closures or global data etc
let ftd_utils = {
    resolve_reference: function (value, reference, data, obj) {
        if (value instanceof Object) {
            let result = value instanceof Array ? [] : {};
            for (var key of Object.keys(value)) {
                if (((typeof value[key]) === "object") && (reference[key] !== undefined)) {
                    result[key] = ftd_utils.resolve_reference(value[key], reference[key], data);
                } else if (reference[key] !== undefined && reference[key] !== null) {
                    result[key] = data[reference[key]]["value"] !== undefined ? data[reference[key]].value : value[key];
                } else {
                    result[key] = (value[key] === "$VALUE" && obj.value !== undefined) ? obj.value : value[key];
                }
            }
            for (var key of Object.keys(reference)) {
                if (value[key] === undefined && data[reference[key]]["value"] !== undefined) {
                    result[key] = data[reference[key]].value;
                }
            }
            return result;
        } else if (reference!== null && reference!== undefined && data[reference]["value"] !== undefined) {
            return data[reference]["value"];
        } else {
            return (value === "$VALUE" && obj.value !== undefined) ? obj.value : value;
        }
    },

    is_visible: function (id, affected_id) {
        return (document.querySelector(`[data-id="${affected_id}:${id}"]`).style.display !== "none");
    },

    box_shadow_value_null: function (value) {
        return (value === "0px 0px 0px 0px") ? null : value;
    },

    box_shadow_value: function (parameter, data_id, value) {
        let current_value  = document.querySelector(`[data-id="${data_id}"]`).style.getPropertyValue('box-shadow');
        if (current_value.length === 0) {
            current_value = "0px 0px 0px 0px";
        }
        let first_split = current_value.split(') ');
        if (first_split.length === 1) {
            first_split.unshift('');
        } else {
            first_split[0] = `${first_split[0]})`;
        }
        if (parameter === "shadow-color") {
            if (value === null) {
                return ftd_utils.box_shadow_value_null(first_split[1].trim());
            }
            first_split[0] = value;
            return ftd_utils.box_shadow_value_null(first_split.join(' ').trim());
        }
        let second_split =  first_split[1].split(' ');
        if (parameter === "shadow-offset-x") {
            second_split[0] = value !== null ? value : '0px' ;
            first_split[1] = second_split.join(' ');
            return ftd_utils.box_shadow_value_null(first_split.join(' ').trim());
        }
        if (parameter === "shadow-offset-y") {
            second_split[1] = value !== null ? value : '0px' ;
            first_split[1] = second_split.join(' ');
            return ftd_utils.box_shadow_value_null(first_split.join(' ').trim());
        }
        if (parameter === "shadow-blur") {
            second_split[2] = value !== null ? value : '0px' ;
            first_split[1] = second_split.join(' ');
            return ftd_utils.box_shadow_value_null(first_split.join(' ').trim());
        }
        if (parameter === "shadow-size") {
            second_split[3] = value !== null ? value : '0px' ;
            first_split[1] = second_split.join(' ');
            return ftd_utils.box_shadow_value_null(first_split.join(' ').trim());
        }
    },

    align_value: function (data_id, value) {
        let current_position  = document.querySelector(`[data-id="${data_id}"]`).style.getPropertyValue('position');
        if (current_position === "fixed" || current_position === "absolute") {
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('transform', null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('right', null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('bottom', null);
            if (value === "center") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', '50%');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', '50%');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('transform', 'translate(-50%,-50%)');
            } else if (value === "top") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', '50%');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('transform', 'translateX(-50%)');
            } else if (value === "left") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', '50%');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('transform', 'translateY(-50%)');
            } else if (value === "right") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('right', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', '50%');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('transform', 'translate(-50%)');
            } else if (value === "bottom") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', '50%');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('bottom', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('transform', 'translateX(-50%)');
            } else if (value === "top-left") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', '0');
            } else if (value === "top-right") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('right', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('top', '0');
            } else if (value === "bottom-left") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('left', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('bottom', '0');
            } else if (value === "bottom-right") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('right', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('bottom', '0');
            }
        } else {
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom',null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top',null);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-left',null);
            if (value === "center") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'center');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', 'auto');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top', 'auto');
            } else if (value === "top") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'center');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', 'auto');
            } else if (value === "left") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'flex-start');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', 'auto');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top', 'auto');
            } else if (value === "right") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'flex-end');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', 'auto');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top', 'auto');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-left', 'auto');
            } else if (value === "bottom") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'center');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top', 'auto');
            } else if (value === "top-left") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'flex-start');
            } else if (value === "top-right") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'flex-end');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-left', 'auto');
            } else if (value === "bottom-left") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'flex-start');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top', 'auto');
            } else if (value === "bottom-right") {
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('align-self', 'flex-end');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-bottom', '0');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-top', 'auto');
                document.querySelector(`[data-id="${data_id}"]`).style.setProperty('margin-left', 'auto');
            }
        }

    },

    line_clamp: function (data_id, value) {
        let doc = document.querySelector(`[data-id="${data_id}"]`);
        if (value == null) {
            doc.style.setProperty('display', null);
            doc.style.setProperty('overflow', null);
            doc.style.setProperty('-webkit-line-clamp', null);
            doc.style.setProperty('-webkit-box-orient', null);
        } else {
            doc.style.setProperty('display', '-webkit-box');
            doc.style.setProperty('overflow', 'hidden');
            doc.style.setProperty('-webkit-line-clamp', value);
            doc.style.setProperty('-webkit-box-orient', 'vertical');
        }
    },

    background_image: function (data_id, value) {
        let background_repeat = document.querySelector(`[data-id="${data_id}"]`).style.getPropertyValue('background-repeat');
        let doc = document.querySelector(`[data-id="${data_id}"]`);
        if (value == null) {
            doc.style.setProperty('background-image', null);
            doc.style.setProperty('background-size', null);
            doc.style.setProperty('background-position', null);
        } else {
            doc.style.setProperty('background-image', `url(${value})`);
            if (background_repeat.length === 0) {
                doc.style.setProperty('background-size', 'cover');
                doc.style.setProperty('background-position', 'center');
            }
        }
    },

    background_repeat: function (data_id, value) {
        let doc = document.querySelector(`[data-id="${data_id}"]`);
        if (value == null) {
            doc.style.setProperty('background-repeat', null);
            doc.style.setProperty('background-size', 'cover');
            doc.style.setProperty('background-position', 'center');
        } else {
            doc.style.setProperty('background-repeat', 'repeat');
            doc.style.setProperty('background-size', null);
            doc.style.setProperty('background-position', null);
        }
    },

    first_child_styling: function (data_id) {
        let parent = document.querySelector(`[data-id="${data_id}"]`).parentElement;
        if (parent.dataset.spacing !== undefined) {
            let spacing = parent.dataset.spacing.split(":");
            let property = spacing[0].trim();
            let value = spacing[1].trim();
            let first_child = true;
            for (let i = 0; i < parent.children.length; i++) {
                if (!first_child) {
                    parent.children[i].style.setProperty(property, value);
                } else if (parent.children[i].style.display !== 'none') {
                    parent.children[i].style.setProperty(property, null);
                    first_child = false;
                }
            }
        }
    },


    set_style: function (parameter, data_id, value, important) {
        if (["shadow-offset-x", "shadow-offset-y", "shadow-size", "shadow-blur", "shadow-color"].includes(parameter)) {
            let box_shadow_value = ftd_utils.box_shadow_value(parameter,data_id, value);
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty('box-shadow', box_shadow_value);
        } else if (parameter === "align" || parameter === "position") {
            ftd_utils.align_value(data_id, value);
        } else if (parameter === "line-clamp") {
            ftd_utils.line_clamp(data_id, value);
        } else if (parameter === "background-image") {
            ftd_utils.background_image(data_id, value);
        } else if (parameter === "background-repeat") {
            ftd_utils.background_repeat(data_id, value);
        } else if (important) {
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty(`${parameter}`, value, 'important');
        } else {
            document.querySelector(`[data-id="${data_id}"]`).style.setProperty(`${parameter}`, value);
        }
    },

    isJson: function(str) {
        try {
            JSON.parse(str);
        } catch (e) {
            return false;
        }
        return true;
    },

    getString: (function() {
        var DIV = document.createElement("div");

        if ('outerHTML' in DIV)
            return function(node) {
                return node.outerHTML;
            };

        return function(node) {
            var div = DIV.cloneNode();
            div.appendChild(node.cloneNode(true));
            return div.innerHTML;
        };

    })(),

    create_dom: function (value,  node) {
        let dom_ids = [];
        let parent_node = node.parentElement;
        if (ftd_utils.isJson(value)) {
            let object = JSON.parse(value);
            for (const idx in object) {
                var new_node = node.cloneNode(true);
                new_node.style.display = null;
                let id = new_node.getAttribute("data-id").replace(":dummy", ",".concat(idx, ":new"));
                new_node.setAttribute("data-id", id);
                dom_ids.push(id);
                parent_node.innerHTML += ftd_utils.getString(new_node).replace("$loop$", object[idx]);
            }
        } else {
            var new_node = node.cloneNode(true);
            new_node.style.display = null;
            let id = new_node.getAttribute("data-id").replace(":dummy", ",0:new");
            new_node.setAttribute("data-id", id);
            dom_ids.push(id);
            parent_node.innerHTML += ftd_utils.getString(new_node).replace("$loop$", value);
        }
        return dom_ids;
    },

    remove_nodes: function (nodes, id) {
        for (const node in nodes) {
            console.log(`${nodes[node]}:${id}`);
            document.querySelector(`[data-id="${nodes[node]}:${id}"]`).remove();
        }
    },

    handle_action: function (id, target, value, data, ftd_external_children) {
        data[target].value = value.toString();

        let dependencies = data[target].dependencies;
        for (const dependency in dependencies) {
            if (!dependencies.hasOwnProperty(dependency)) {
                continue;
            }
            let json_dependencies = JSON.parse(dependencies[dependency]);
            var styles_edited = [];
            for (const index in json_dependencies) {
                let json_dependency = json_dependencies[index];
                if (json_dependency.dependency_type === "Value") {
                    if (dependency.endsWith(':dummy')) {
                        let dummy_node = document.querySelector(`[data-id="${dependency}:${id}"]`);
                        let dom_ids = ftd_utils.create_dom(data[target].value, dummy_node);
                        ftd_utils.remove_nodes(Object.keys(dependencies).filter(s => !s.endsWith(':dummy')), id);
                        let deps = {};
                        for (const dom_id in dom_ids) {
                            let id_without_main = dom_ids[dom_id].substring(0, dom_ids[dom_id].length - `:${id}`.length )
                            deps[id_without_main] = dependencies[dependency];
                        }
                        deps[dependency] = dependencies[dependency];
                        data[target].dependencies = deps;
                    } else {
                        document.querySelector(`[data-id="${dependency}:${id}"]`).innerText = data[target].value;
                    }
                } else if (json_dependency.dependency_type === "Visible") {
                    let display = "none";
                    if ((data[target].value === json_dependency.condition)
                        || (json_dependency.condition === "$IsNull$" && data[target].value.trim().length === 0)
                        || (json_dependency.condition === "$IsNotNull$" && data[target].value.trim().length !== 0)
                    ) {
                        let is_flex = !!document.querySelector(`[data-id="${dependency}:${id}"]`).style.flexDirection.length;
                        let is_grid = !!document.querySelector(`[data-id="${dependency}:${id}"]`).style.gridTemplateAreas.length;
                        let is_webkit = !!document.querySelector(`[data-id="${dependency}:${id}"]`).style.webkitLineClamp.length;
                        if (is_flex) {
                            display = "flex";
                        } else if (is_webkit) {
                            display = "-webkit-box";
                        } else if (is_grid) {
                            display = "grid";
                        } else {
                            display = "block";
                        }
                    }
                    document.querySelector(`[data-id="${dependency}:${id}"]`).style.display = display;
                    ftd_utils.first_child_styling(`${dependency}:${id}`);

                } else if (json_dependency.dependency_type === "Style") {
                    if (data[target].value === json_dependency.condition) {
                        for (const parameter in json_dependency.parameters) {
                            let value = json_dependency.parameters[`${parameter}`].value.value;
                            let important = json_dependency.parameters[`${parameter}`].value.important;
                            ftd_utils.set_style(parameter, `${dependency}:${id}`, value, important);
                            if (!styles_edited.includes(parameter)) {
                                styles_edited.push(parameter);
                            }
                        }
                    } else {
                        for (const parameter in json_dependency.parameters) {
                            let default_value = json_dependency.parameters[`${parameter}`].default;
                            if (!styles_edited.includes(parameter)) {
                                if (default_value === null) {
                                    if (["border-left-width", "border-right-width", "border-top-width", "border-bottom-width"].includes(parameter)) {
                                        default_value = "0px";
                                        document.querySelector(`[data-id="${dependency}:${id}"]`).style[`${parameter}`] = default_value;
                                    } else {
                                        ftd_utils.set_style(parameter, `${dependency}:${id}`, default_value, false);
                                    }
                                } else {
                                    let value = default_value.value;
                                    let important = default_value.important;
                                    ftd_utils.set_style(parameter, `${dependency}:${id}`, value, important);
                                }
                            }
                        }
                    }
                }
            }
        }
        this.external_children_replace(id, ftd_external_children)
    },

    external_children_replace: function (id, ftd_external_children) {
        if (ftd_external_children[id] === undefined) {
            return;
        }
        let external_children = ftd_external_children[id];
        let external_children_placed = [];
        for (const object in external_children) {
            if (!external_children.hasOwnProperty(object)) {
                continue;
            }

            let conditions = external_children[object];
            for (const idx in conditions) {
                if (!conditions.hasOwnProperty(idx)) {
                    continue;
                }

                let condition = conditions[idx].condition;
                let set_at = conditions[idx].set_at;
                let display = true;
                for (const i in condition) {
                    if (!condition.hasOwnProperty(i)) {
                        continue;
                    }

                    display &= ftd_utils.is_visible(id, conditions[idx].condition[i])
                    if (!display) {
                        break;
                    }
                }
                if (display && !external_children_placed.includes(object)) {
                    console.log(`${object}:${id}::: ${set_at}:${id}`);
                    let get_element_set_at = document.querySelector(`[data-id="${set_at}:${id}"]`);
                    let objects_to_set = document.querySelectorAll(`[data-ext-id="${object}:${id}"]`);
                    for (let i = 0; i < objects_to_set.length; i++) {
                        let object_to_set = objects_to_set[i];
                        let parent = object_to_set.parentElement;
                        if (parent !== get_element_set_at) {
                            get_element_set_at.appendChild(object_to_set);
                        }
                    }
                    external_children_placed.push(object);
                }
            }

        }
    }
};

window.ftd = (function () {
    let ftd_data = {};
    let ftd_external_children = {};

    function handle_event(evt, id, action, obj) {
        let act = action["action"];
        let data = ftd_data[id];
        if (act === "stop-propagation") {
            evt.stopPropagation();
        } else if (act === "prevent-default") {
            evt.preventDefault();
        } else if (act === "toggle") {
            let target = action["target"];
            exports.set_bool(id, target, data[target].value !== 'true');
        } else if (act === "message-host") {
            if (action["parameters"].data !== undefined) {
                let value = JSON.parse(action["parameters"].data[0].value);
                let reference = JSON.parse(action["parameters"].data[0].reference);
                let data = ftd_utils.resolve_reference(value, reference, ftd_data[id], obj);
                let func = data.function.trim().replaceAll("-", "_");
                window[func](id, data, reference);
            } else {
                let target = action["target"].trim().replaceAll("-", "_");
                window[target](id);
            }
        } else if (act === "increment") {
            let target = action["target"];
            let increment = 1;
            if (action["parameters"].by !== undefined) {
                let by_value = action["parameters"].by[0].value;
                let by_reference = action["parameters"].by[0].reference;
                increment = parseInt(ftd_utils.resolve_reference(by_value, by_reference, ftd_data[id], obj));
            }
            let clamp_max = undefined;
            let clamp_min = undefined;
            if (action["parameters"]["clamp"] !== undefined) {
                let clamp_value = action["parameters"]["clamp"];
                if (clamp_value.length === 1) {
                    clamp_max = parseInt(ftd_utils.resolve_reference(clamp_value[0].value, clamp_value[0].reference, ftd_data[id], obj));
                }
                if (clamp_value.length === 2) {
                    clamp_min = parseInt(ftd_utils.resolve_reference(clamp_value[0].value, clamp_value[0].reference, ftd_data[id], obj));
                    clamp_max = parseInt(ftd_utils.resolve_reference(clamp_value[1].value, clamp_value[1].reference, ftd_data[id], obj));
                }
            }
            exports.increment_decrement_value(id, target, increment, clamp_min, clamp_max);

        } else if (act === "decrement") {
            let target = action["target"];
            let decrement = -1;
            if (action["parameters"].by !== undefined) {
                let by_value = action["parameters"].by[0].value;
                let by_reference = action["parameters"].by[0].reference;
                decrement = -parseInt(ftd_utils.resolve_reference(by_value, by_reference, ftd_data[id], obj));
            }

            let clamp_max = undefined;
            let clamp_min = undefined;
            if (action["parameters"]["clamp"] !== undefined) {
                let clamp_value = action["parameters"]["clamp"];
                if (clamp_value.length === 1) {
                    clamp_max = parseInt(ftd_utils.resolve_reference(clamp_value[0].value, clamp_value[0].reference, ftd_data[id], obj));
                }
                if (clamp_value.length === 2) {
                    clamp_min = parseInt(ftd_utils.resolve_reference(clamp_value[0].value, clamp_value[0].reference, ftd_data[id], obj));
                    clamp_max = parseInt(ftd_utils.resolve_reference(clamp_value[1].value, clamp_value[1].reference, ftd_data[id], obj));
                }
            }

            exports.increment_decrement_value(id, target, decrement, clamp_min, clamp_max);
        } else if (act === "set-value") {
            let target = action["target"];
            let value_data = action["parameters"].value[0];
            let value = ftd_utils.resolve_reference(value_data.value, value_data.reference, ftd_data[id], obj)
            if (action["parameters"].value[1].value === "integer") {
                value = parseInt(value);
            } else if (action["parameters"].value[1].value === "decimal") {
                value = parseFloat(value);
            } else if (action["parameters"].value[1].value === "boolean") {
                value = (value === "true");
            }

            let data = ftd_data[id];
            ftd_utils.handle_action(id, target, value, data, ftd_external_children);

        } else if (act === "insert") {
            let target = action["target"];
            let value = undefined;
            if (action["parameters"].value !== undefined) {
                let insert_value = action["parameters"].value[0].value;
                let insert_reference = action["parameters"].value[0].reference;
                value = ftd_utils.resolve_reference(insert_value, insert_reference, ftd_data[id], obj);
            }
            let at = undefined;
            if (action["parameters"].at !== undefined) {
                let at_value = action["parameters"].at[0].value;
                let at_reference = action["parameters"].at[0].reference;
                at = ftd_utils.resolve_reference(at_value, at_reference, ftd_data[id], obj);
            }

            exports.insert_value(id, target, value, at);

        } else if (act === "clear") {
            let target = action["target"];
            let data = ftd_data[id];
            let value = "";
            if (ftd_utils.isJson(data[target].value)) {
                let list = [];
                value = JSON.stringify(list);
            }
            ftd_utils.handle_action(id, target, value, data, ftd_external_children);
        } else {
            console.log("unknown action:", act);
            return;
        }

    }

    let exports = {};

    exports.handle_event = function (evt, id, event, obj) {
        console.log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(evt, id, actions[action], obj)
        }
    }

    exports.increment_decrement_value = function (id, variable, increment_by, clamp_min, clamp_max) {
        let data = ftd_data[id];

        if (!data[variable]) {
            console.log(variable, "is not in data, ignoring");
            return;
        }

        let value = parseInt(data[variable].value);
        value += increment_by;

        if (clamp_max !== undefined) {
            let min = (clamp_min === undefined) ? 0: clamp_min
            if (clamp_max < value) {
                value = min;
            }
            if (clamp_min > value) {
                value = clamp_max;
            }
        }

        ftd_utils.handle_action(id, variable, value, data, ftd_external_children);
    }

    exports.insert_value = function (id, target, value, at) {
        let data = ftd_data[id];

        if (!data[target]) {
            console.log(target, "is not in data, ignoring");
            return;
        }

        let list = data[target].value;
        if (ftd_utils.isJson(list)) {
            list = JSON.parse(list);
        } else {
            console.log(list, "is not list, ignoring");
            return;
        }

        if (value === undefined || value.trim() === "") {
            console.log("Nothing to insert in ", list);
            return;
        }

        if (at !== undefined && at === "end") {
            list.push(value);
        } else if (at !== undefined && at === "start") {
            list.unshift(value);
        } else {
            list.push(value);
        }

        ftd_utils.handle_action(id, target, JSON.stringify(list), data, ftd_external_children);
    }

    exports.set_bool = function (id, variable, value) {
        let data = ftd_data[id];

        if (!data[variable]) {
            console.log(variable, "is not in data, ignoring");
            return;
        }

        ftd_utils.handle_action(id, variable, value, data, ftd_external_children);
    }

    exports.set_string = function (id, variable, value) {
        let data = ftd_data[id];

        if (!data[variable]) {
            console.log(variable, "is not in data, ignoring");
            return;
        }

        ftd_utils.handle_action(id, variable, value, data);
    }

    exports.get_value = function (id, variable) {
        let data = ftd_data[id];

        if (!data[variable]) {
            console.log(variable, "is not in data, ignoring");
            return;
        }
        return data[variable].value;
    }

    exports.set_multi_value = function (id, list) {
        for (const idx in list) {
            if (!list.hasOwnProperty(idx)) {
                continue;
            }

            let item = list[idx];
            let [variable, value] = item;
            this.set_bool(id, variable, value);
        }
    }

    exports.init = function (id, data, external_children) {
        ftd_data[id] = data;
        ftd_external_children[id] = external_children;
    }

    exports.set_bool_for_all = function (variable, value) {
        for (let id in ftd_data) {
            if (!ftd_data.hasOwnProperty(id)) {
                continue;
            }

            exports.set_bool(id, variable, value)
        }
    }

    exports.set_string_for_all = function (variable, value) {
        for (let id in ftd_data) {
            if (!ftd_data.hasOwnProperty(id)) {
                continue;
            }

            exports.set_string(id, variable, value)
        }
    }

    return exports;
})();

function console_print(id, data) {
    console.log(data);
}
