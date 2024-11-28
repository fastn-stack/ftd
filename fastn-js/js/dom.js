let fastn_dom = {};

fastn_dom.styleClasses = "";

fastn_dom.InternalClass = {
    FT_COLUMN: "ft_column",
    FT_ROW: "ft_row",
    FT_FULL_SIZE: "ft_full_size",
};

fastn_dom.codeData = {
    availableThemes: {},
    addedCssFile: [],
};

fastn_dom.externalCss = new Set();
fastn_dom.externalJs = new Set();

// Todo: Object (key, value) pair (counter type key)
fastn_dom.webComponent = [];

fastn_dom.commentNode = "comment";
fastn_dom.wrapperNode = "wrapper";
fastn_dom.commentMessage = "***FASTN***";
fastn_dom.webComponentArgument = "args";

fastn_dom.classes = {};
fastn_dom.unsanitised_classes = {};
fastn_dom.class_count = 0;
fastn_dom.propertyMap = {
    "align-items": "ali",
    "align-self": "as",
    "background-color": "bgc",
    "background-image": "bgi",
    "background-position": "bgp",
    "background-repeat": "bgr",
    "background-size": "bgs",
    "border-bottom-color": "bbc",
    "border-bottom-left-radius": "bblr",
    "border-bottom-right-radius": "bbrr",
    "border-bottom-style": "bbs",
    "border-bottom-width": "bbw",
    "border-color": "bc",
    "border-left-color": "blc",
    "border-left-style": "bls",
    "border-left-width": "blw",
    "border-radius": "br",
    "border-right-color": "brc",
    "border-right-style": "brs",
    "border-right-width": "brw",
    "border-style": "bs",
    "border-top-color": "btc",
    "border-top-left-radius": "btlr",
    "border-top-right-radius": "btrr",
    "border-top-style": "bts",
    "border-top-width": "btw",
    "border-width": "bw",
    bottom: "b",
    color: "c",
    shadow: "sh",
    "text-shadow": "tsh",
    cursor: "cur",
    display: "d",
    download: "dw",
    "flex-wrap": "fw",
    "font-style": "fst",
    "font-weight": "fwt",
    gap: "g",
    height: "h",
    "justify-content": "jc",
    left: "l",
    link: "lk",
    "link-color": "lkc",
    margin: "m",
    "margin-bottom": "mb",
    "margin-horizontal": "mh",
    "margin-left": "ml",
    "margin-right": "mr",
    "margin-top": "mt",
    "margin-vertical": "mv",
    "max-height": "mxh",
    "max-width": "mxw",
    "min-height": "mnh",
    "min-width": "mnw",
    opacity: "op",
    overflow: "o",
    "overflow-x": "ox",
    "overflow-y": "oy",
    "object-fit": "of",
    padding: "p",
    "padding-bottom": "pb",
    "padding-horizontal": "ph",
    "padding-left": "pl",
    "padding-right": "pr",
    "padding-top": "pt",
    "padding-vertical": "pv",
    position: "pos",
    resize: "res",
    role: "rl",
    right: "r",
    sticky: "s",
    "text-align": "ta",
    "text-decoration": "td",
    "text-transform": "tt",
    top: "t",
    width: "w",
    "z-index": "z",
    "-webkit-box-orient": "wbo",
    "-webkit-line-clamp": "wlc",
    "backdrop-filter": "bdf",
    "mask-image": "mi",
    "-webkit-mask-image": "wmi",
    "mask-size": "ms",
    "-webkit-mask-size": "wms",
    "mask-repeat": "mre",
    "-webkit-mask-repeat": "wmre",
    "mask-position": "mp",
    "-webkit-mask-position": "wmp",
    "fetch-priority": "ftp",
};

// dynamic-class-css.md
fastn_dom.getClassesAsString = function () {
    return `<style id="styles">
    ${fastn_dom.getClassesAsStringWithoutStyleTag()}
    </style>`;
};

fastn_dom.getClassesAsStringWithoutStyleTag = function () {
    let classes = Object.entries(fastn_dom.classes).map((entry) => {
        return getClassAsString(entry[0], entry[1]);
    });

    /*.ft_text {
        padding: 0;
    }*/
    return classes.join("\n\t");
};

function getClassAsString(className, obj) {
    if (typeof obj.value === "object" && obj.value !== null) {
        let value = "";
        for (let key in obj.value) {
            if (obj.value[key] === undefined || obj.value[key] === null) {
                continue;
            }
            value = `${value} ${key}: ${obj.value[key]}${
                key === "color" ? " !important" : ""
            };`;
        }
        return `${className} { ${value} }`;
    } else {
        return `${className} { ${obj.property}: ${obj.value}${
            obj.property === "color" ? " !important" : ""
        }; }`;
    }
}

fastn_dom.ElementKind = {
    Row: 0,
    Column: 1,
    Integer: 2,
    Decimal: 3,
    Boolean: 4,
    Text: 5,
    Image: 6,
    IFrame: 7,
    // To create parent for dynamic DOM
    Comment: 8,
    CheckBox: 9,
    TextInput: 10,
    ContainerElement: 11,
    Rive: 12,
    Document: 13,
    Wrapper: 14,
    Code: 15,
    // Note: This is called internally, it gives `code` as tagName. This is used
    // along with the Code: 15.
    CodeChild: 16,
    // Note: 'arguments' cant be used as function parameter name bcoz it has
    // internal usage in js functions.
    WebComponent: (webcomponent, args) => {
        return [17, [webcomponent, args]];
    },
    Video: 18,
    Audio: 19,
};

fastn_dom.PropertyKind = {
    Color: 0,
    IntegerValue: 1,
    StringValue: 2,
    DecimalValue: 3,
    BooleanValue: 4,
    Width: 5,
    Padding: 6,
    Height: 7,
    Id: 8,
    BorderWidth: 9,
    BorderStyle: 10,
    Margin: 11,
    Background: 12,
    PaddingHorizontal: 13,
    PaddingVertical: 14,
    PaddingLeft: 15,
    PaddingRight: 16,
    PaddingTop: 17,
    PaddingBottom: 18,
    MarginHorizontal: 19,
    MarginVertical: 20,
    MarginLeft: 21,
    MarginRight: 22,
    MarginTop: 23,
    MarginBottom: 24,
    Role: 25,
    ZIndex: 26,
    Sticky: 27,
    Top: 28,
    Bottom: 29,
    Left: 30,
    Right: 31,
    Overflow: 32,
    OverflowX: 33,
    OverflowY: 34,
    Spacing: 35,
    Wrap: 36,
    TextTransform: 37,
    TextIndent: 38,
    TextAlign: 39,
    LineClamp: 40,
    Opacity: 41,
    Cursor: 42,
    Resize: 43,
    MinHeight: 44,
    MaxHeight: 45,
    MinWidth: 46,
    MaxWidth: 47,
    WhiteSpace: 48,
    BorderTopWidth: 49,
    BorderBottomWidth: 50,
    BorderLeftWidth: 51,
    BorderRightWidth: 52,
    BorderRadius: 53,
    BorderTopLeftRadius: 54,
    BorderTopRightRadius: 55,
    BorderBottomLeftRadius: 56,
    BorderBottomRightRadius: 57,
    BorderStyleVertical: 58,
    BorderStyleHorizontal: 59,
    BorderLeftStyle: 60,
    BorderRightStyle: 61,
    BorderTopStyle: 62,
    BorderBottomStyle: 63,
    BorderColor: 64,
    BorderLeftColor: 65,
    BorderRightColor: 66,
    BorderTopColor: 67,
    BorderBottomColor: 68,
    AlignSelf: 69,
    Classes: 70,
    Anchor: 71,
    Link: 72,
    Children: 73,
    OpenInNewTab: 74,
    TextStyle: 75,
    Region: 76,
    AlignContent: 77,
    Display: 78,
    Checked: 79,
    Enabled: 80,
    TextInputType: 81,
    Placeholder: 82,
    Multiline: 83,
    DefaultTextInputValue: 84,
    Loading: 85,
    Src: 86,
    YoutubeSrc: 87,
    Code: 88,
    ImageSrc: 89,
    Alt: 90,
    DocumentProperties: {
        MetaTitle: 91,
        MetaOGTitle: 92,
        MetaTwitterTitle: 93,
        MetaDescription: 94,
        MetaOGDescription: 95,
        MetaTwitterDescription: 96,
        MetaOGImage: 97,
        MetaTwitterImage: 98,
        MetaThemeColor: 99,
        MetaFacebookDomainVerification: 100,
    },
    Shadow: 101,
    CodeTheme: 102,
    CodeLanguage: 103,
    CodeShowLineNumber: 104,
    Css: 105,
    Js: 106,
    LinkRel: 107,
    InputMaxLength: 108,
    Favicon: 109,
    Fit: 110,
    VideoSrc: 111,
    Autoplay: 112,
    Poster: 113,
    Loop: 114,
    Controls: 115,
    Muted: 116,
    LinkColor: 117,
    TextShadow: 118,
    Selectable: 119,
    BackdropFilter: 120,
    Mask: 121,
    TextInputValue: 122,
    FetchPriority: 123,
    Download: 124,
    SrcDoc: 125,
};

fastn_dom.Loading = {
    Lazy: "lazy",
    Eager: "eager",
};

fastn_dom.LinkRel = {
    NoFollow: "nofollow",
    Sponsored: "sponsored",
    Ugc: "ugc",
};

fastn_dom.TextInputType = {
    Text: "text",
    Email: "email",
    Password: "password",
    Url: "url",
    DateTime: "datetime",
    Date: "date",
    Time: "time",
    Month: "month",
    Week: "week",
    Color: "color",
    File: "file",
};

fastn_dom.AlignContent = {
    TopLeft: "top-left",
    TopCenter: "top-center",
    TopRight: "top-right",
    Right: "right",
    Left: "left",
    Center: "center",
    BottomLeft: "bottom-left",
    BottomRight: "bottom-right",
    BottomCenter: "bottom-center",
};

fastn_dom.Region = {
    H1: "h1",
    H2: "h2",
    H3: "h3",
    H4: "h4",
    H5: "h5",
    H6: "h6",
};

fastn_dom.Anchor = {
    Window: [1, "fixed"],
    Parent: [2, "absolute"],
    Id: (value) => {
        return [3, value];
    },
};

fastn_dom.DeviceData = {
    Desktop: "desktop",
    Mobile: "mobile",
};

fastn_dom.TextStyle = {
    Underline: "underline",
    Italic: "italic",
    Strike: "line-through",
    Heavy: "900",
    Extrabold: "800",
    Bold: "700",
    SemiBold: "600",
    Medium: "500",
    Regular: "400",
    Light: "300",
    ExtraLight: "200",
    Hairline: "100",
};

fastn_dom.Resizing = {
    FillContainer: "100%",
    HugContent: "fit-content",
    Auto: "auto",
    Fixed: (value) => {
        return value;
    },
};

fastn_dom.Spacing = {
    SpaceEvenly: [1, "space-evenly"],
    SpaceBetween: [2, "space-between"],
    SpaceAround: [3, "space-around"],
    Fixed: (value) => {
        return [4, value];
    },
};

fastn_dom.BorderStyle = {
    Solid: "solid",
    Dashed: "dashed",
    Dotted: "dotted",
    Double: "double",
    Ridge: "ridge",
    Groove: "groove",
    Inset: "inset",
    Outset: "outset",
};

fastn_dom.Fit = {
    none: "none",
    fill: "fill",
    contain: "contain",
    cover: "cover",
    scaleDown: "scale-down",
};

fastn_dom.FetchPriority = {
    auto: "auto",
    high: "high",
    low: "low",
};

fastn_dom.Overflow = {
    Scroll: "scroll",
    Visible: "visible",
    Hidden: "hidden",
    Auto: "auto",
};

fastn_dom.Display = {
    Block: "block",
    Inline: "inline",
    InlineBlock: "inline-block",
};

fastn_dom.AlignSelf = {
    Start: "start",
    Center: "center",
    End: "end",
};

fastn_dom.TextTransform = {
    None: "none",
    Capitalize: "capitalize",
    Uppercase: "uppercase",
    Lowercase: "lowercase",
    Inherit: "inherit",
    Initial: "initial",
};

fastn_dom.TextAlign = {
    Start: "start",
    Center: "center",
    End: "end",
    Justify: "justify",
};

fastn_dom.Cursor = {
    None: "none",
    Default: "default",
    ContextMenu: "context-menu",
    Help: "help",
    Pointer: "pointer",
    Progress: "progress",
    Wait: "wait",
    Cell: "cell",
    CrossHair: "crosshair",
    Text: "text",
    VerticalText: "vertical-text",
    Alias: "alias",
    Copy: "copy",
    Move: "move",
    NoDrop: "no-drop",
    NotAllowed: "not-allowed",
    Grab: "grab",
    Grabbing: "grabbing",
    EResize: "e-resize",
    NResize: "n-resize",
    NeResize: "ne-resize",
    SResize: "s-resize",
    SeResize: "se-resize",
    SwResize: "sw-resize",
    Wresize: "w-resize",
    Ewresize: "ew-resize",
    NsResize: "ns-resize",
    NeswResize: "nesw-resize",
    NwseResize: "nwse-resize",
    ColResize: "col-resize",
    RowResize: "row-resize",
    AllScroll: "all-scroll",
    ZoomIn: "zoom-in",
    ZoomOut: "zoom-out",
};

fastn_dom.Resize = {
    Vertical: "vertical",
    Horizontal: "horizontal",
    Both: "both",
};

fastn_dom.WhiteSpace = {
    Normal: "normal",
    NoWrap: "nowrap",
    Pre: "pre",
    PreLine: "pre-line",
    PreWrap: "pre-wrap",
    BreakSpaces: "break-spaces",
};

fastn_dom.BackdropFilter = {
    Blur: (value) => {
        return [1, value];
    },
    Brightness: (value) => {
        return [2, value];
    },
    Contrast: (value) => {
        return [3, value];
    },
    Grayscale: (value) => {
        return [4, value];
    },
    Invert: (value) => {
        return [5, value];
    },
    Opacity: (value) => {
        return [6, value];
    },
    Sepia: (value) => {
        return [7, value];
    },
    Saturate: (value) => {
        return [8, value];
    },
    Multi: (value) => {
        return [9, value];
    },
};

fastn_dom.BackgroundStyle = {
    Solid: (value) => {
        return [1, value];
    },
    Image: (value) => {
        return [2, value];
    },
    LinearGradient: (value) => {
        return [3, value];
    },
};

fastn_dom.BackgroundRepeat = {
    Repeat: "repeat",
    RepeatX: "repeat-x",
    RepeatY: "repeat-y",
    NoRepeat: "no-repeat",
    Space: "space",
    Round: "round",
};

fastn_dom.BackgroundSize = {
    Auto: "auto",
    Cover: "cover",
    Contain: "contain",
    Length: (value) => {
        return value;
    },
};

fastn_dom.BackgroundPosition = {
    Left: "left",
    Right: "right",
    Center: "center",
    LeftTop: "left top",
    LeftCenter: "left center",
    LeftBottom: "left bottom",
    CenterTop: "center top",
    CenterCenter: "center center",
    CenterBottom: "center bottom",
    RightTop: "right top",
    RightCenter: "right center",
    RightBottom: "right bottom",
    Length: (value) => {
        return value;
    },
};

fastn_dom.LinearGradientDirection = {
    Angle: (value) => {
        return `${value}deg`;
    },
    Turn: (value) => {
        return `${value}turn`;
    },
    Left: "270deg",
    Right: "90deg",
    Top: "0deg",
    Bottom: "180deg",
    TopLeft: "315deg",
    TopRight: "45deg",
    BottomLeft: "225deg",
    BottomRight: "135deg",
};

fastn_dom.FontSize = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${value.get()}px`;
            });
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${value.get()}em`;
            });
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${value.get()}rem`;
            });
        }
        return `${value}rem`;
    },
};

fastn_dom.Length = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}px`;
            });
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}em`;
            });
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}rem`;
            });
        }
        return `${value}rem`;
    },
    Percent: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}%`;
            });
        }
        return `${value}%`;
    },
    Calc: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `calc(${fastn_utils.getStaticValue(value)})`;
            });
        }
        return `calc(${value})`;
    },
    Vh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vh`;
            });
        }
        return `${value}vh`;
    },
    Vw: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vw`;
            });
        }
        return `${value}vw`;
    },
    Dvh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}dvh`;
            });
        }
        return `${value}dvh`;
    },
    Lvh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}lvh`;
            });
        }
        return `${value}lvh`;
    },
    Svh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}svh`;
            });
        }
        return `${value}svh`;
    },

    Vmin: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vmin`;
            });
        }
        return `${value}vmin`;
    },
    Vmax: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vmax`;
            });
        }
        return `${value}vmax`;
    },
    Responsive: (length) => {
        return new PropertyValueAsClosure(() => {
            if (ftd.device.get() === "desktop") {
                return length.get("desktop");
            } else {
                let mobile = length.get("mobile");
                let desktop = length.get("desktop");
                return mobile ? mobile : desktop;
            }
        }, [ftd.device, length]);
    },
};

fastn_dom.Mask = {
    Image: (value) => {
        return [1, value];
    },
    Multi: (value) => {
        return [2, value];
    },
};

fastn_dom.MaskSize = {
    Auto: "auto",
    Cover: "cover",
    Contain: "contain",
    Fixed: (value) => {
        return value;
    },
};

fastn_dom.MaskRepeat = {
    Repeat: "repeat",
    RepeatX: "repeat-x",
    RepeatY: "repeat-y",
    NoRepeat: "no-repeat",
    Space: "space",
    Round: "round",
};

fastn_dom.MaskPosition = {
    Left: "left",
    Right: "right",
    Center: "center",
    LeftTop: "left top",
    LeftCenter: "left center",
    LeftBottom: "left bottom",
    CenterTop: "center top",
    CenterCenter: "center center",
    CenterBottom: "center bottom",
    RightTop: "right top",
    RightCenter: "right center",
    RightBottom: "right bottom",
    Length: (value) => {
        return value;
    },
};

fastn_dom.Event = {
    Click: 0,
    MouseEnter: 1,
    MouseLeave: 2,
    ClickOutside: 3,
    GlobalKey: (val) => {
        return [4, val];
    },
    GlobalKeySeq: (val) => {
        return [5, val];
    },
    Input: 6,
    Change: 7,
    Blur: 8,
    Focus: 9,
};

class PropertyValueAsClosure {
    closureFunction;
    deps;
    constructor(closureFunction, deps) {
        this.closureFunction = closureFunction;
        this.deps = deps;
    }
}

// Node2 -> Intermediate node
// Node -> similar to HTML DOM node (Node2.#node)
class Node2 {
    #node;
    #kind;
    #parent;
    #tagName;
    #rawInnerValue;
    /**
     * This is where we store all the attached closures, so we can free them
     * when we are done.
     */
    #mutables;
    /**
     * This is where we store the extraData related to node. This is
     * especially useful to store data for integrated external library (like
     * rive).
     */
    #extraData;
    #children;
    constructor(parentOrSibiling, kind) {
        this.#kind = kind;
        this.#parent = parentOrSibiling;
        this.#children = [];
        this.#rawInnerValue = null;

        let sibiling = undefined;

        if (parentOrSibiling instanceof ParentNodeWithSibiling) {
            this.#parent = parentOrSibiling.getParent();
            while (this.#parent instanceof ParentNodeWithSibiling) {
                this.#parent = this.#parent.getParent();
            }
            sibiling = parentOrSibiling.getSibiling();
        }

        this.createNode(kind);

        this.#mutables = [];
        this.#extraData = {};
        /*if (!!parent.parent) {
            parent = parent.parent();
        }*/

        if (this.#parent.getNode) {
            this.#parent = this.#parent.getNode();
        }

        if (fastn_utils.isWrapperNode(this.#tagName)) {
            this.#parent = parentOrSibiling;
            return;
        }
        if (sibiling) {
            this.#parent.insertBefore(
                this.#node,
                fastn_utils.nextSibling(sibiling, this.#parent),
            );
        } else {
            this.#parent.appendChild(this.#node);
        }
    }
    createNode(kind) {
        if (kind === fastn_dom.ElementKind.Code) {
            let [node, classes, attributes] = fastn_utils.htmlNode(kind);
            [this.#tagName, this.#node] = fastn_utils.createNodeHelper(
                node,
                classes,
                attributes,
            );
            let codeNode = new Node2(
                this.#node,
                fastn_dom.ElementKind.CodeChild,
            );
            this.#children.push(codeNode);
        } else {
            let [node, classes, attributes] = fastn_utils.htmlNode(kind);
            [this.#tagName, this.#node] = fastn_utils.createNodeHelper(
                node,
                classes,
                attributes,
            );
        }
    }
    getTagName() {
        return this.#tagName;
    }
    getParent() {
        return this.#parent;
    }
    removeAllFaviconLinks() {
        if (doubleBuffering) {
            const links = document.head.querySelectorAll(
                'link[rel="shortcut icon"]',
            );
            links.forEach((link) => {
                link.parentNode.removeChild(link);
            });
        }
    }
    setFavicon(url) {
        if (doubleBuffering) {
            if (url instanceof fastn.recordInstanceClass) url = url.get("src");
            while (true) {
                if (url instanceof fastn.mutableClass) url = url.get();
                else break;
            }

            let link_element = document.createElement("link");
            link_element.rel = "shortcut icon";
            link_element.href = url;

            this.removeAllFaviconLinks();
            document.head.appendChild(link_element);
        }
    }
    updateTextInputValue() {
        if (fastn_utils.isNull(this.#rawInnerValue)) {
            this.attachAttribute("value");
            return;
        }
        if (!ssr && this.#node.tagName.toLowerCase() === "textarea") {
            this.#node.innerHTML = this.#rawInnerValue;
        } else {
            this.attachAttribute("value", this.#rawInnerValue);
        }
    }
    // for attaching inline attributes
    attachAttribute(property, value) {
        // If the value is null, undefined, or false, the attribute will be removed.
        // For example, if attributes like checked, muted, or autoplay have been assigned a "false" value.
        if (fastn_utils.isNull(value)) {
            this.#node.removeAttribute(property);
            return;
        }
        this.#node.setAttribute(property, value);
    }
    removeAttribute(property) {
        this.#node.removeAttribute(property);
    }
    updateTagName(name) {
        if (ssr) {
            this.#node.updateTagName(name);
        } else {
            let newElement = document.createElement(name);
            newElement.innerHTML = this.#node.innerHTML;
            newElement.className = this.#node.className;
            newElement.style = this.#node.style;
            for (var i = 0; i < this.#node.attributes.length; i++) {
                var attr = this.#node.attributes[i];
                newElement.setAttribute(attr.name, attr.value);
            }
            var eventListeners = fastn_utils.getEventListeners(this.#node);
            for (var eventType in eventListeners) {
                newElement[eventType] = eventListeners[eventType];
            }
            this.#parent.replaceChild(newElement, this.#node);
            this.#node = newElement;
        }
    }
    updateToAnchor(url) {
        let node_kind = this.#kind;
        if (ssr) {
            if (node_kind !== fastn_dom.ElementKind.Image) {
                this.updateTagName("a");
                this.attachAttribute("href", url);
            }
            return;
        }
        if (node_kind === fastn_dom.ElementKind.Image) {
            let anchorElement = document.createElement("a");
            anchorElement.href = url;
            anchorElement.appendChild(this.#node);
            this.#parent.appendChild(anchorElement);
            this.#node = anchorElement;
        } else {
            this.updateTagName("a");
            this.#node.href = url;
        }
    }
    updatePositionForNodeById(node_id, value) {
        if (!ssr) {
            const target_node = fastnVirtual.root.querySelector(
                `[id="${node_id}"]`,
            );
            if (!fastn_utils.isNull(target_node))
                target_node.style["position"] = value;
        }
    }
    updateParentPosition(value) {
        if (ssr) {
            let parent = this.#parent;
            if (parent.style) parent.style["position"] = value;
        }
        if (!ssr) {
            let current_node = this.#node;
            if (current_node) {
                let parent_node = current_node.parentNode;
                parent_node.style["position"] = value;
            }
        }
    }
    updateMetaTitle(value) {
        if (!ssr && doubleBuffering) {
            if (!fastn_utils.isNull(value)) window.document.title = value;
        }
    }
    addMetaTagByName(name, value) {
        if (value === null || value === undefined) {
            this.removeMetaTagByName(name);
            return;
        }
        if (!ssr && doubleBuffering) {
            const metaTag = window.document.createElement("meta");
            metaTag.setAttribute("name", name);
            metaTag.setAttribute("content", value);
            document.head.appendChild(metaTag);
        }
    }
    addMetaTagByProperty(property, value) {
        if (value === null || value === undefined) {
            this.removeMetaTagByProperty(property);
            return;
        }
        if (!ssr && doubleBuffering) {
            const metaTag = window.document.createElement("meta");
            metaTag.setAttribute("property", property);
            metaTag.setAttribute("content", value);
            document.head.appendChild(metaTag);
        }
    }
    removeMetaTagByName(name) {
        if (!ssr && doubleBuffering) {
            const metaTags = document.getElementsByTagName("meta");
            for (let i = 0; i < metaTags.length; i++) {
                const metaTag = metaTags[i];
                if (metaTag.getAttribute("name") === name) {
                    metaTag.remove();
                    break;
                }
            }
        }
    }
    removeMetaTagByProperty(property) {
        if (!ssr && doubleBuffering) {
            const metaTags = document.getElementsByTagName("meta");
            for (let i = 0; i < metaTags.length; i++) {
                const metaTag = metaTags[i];
                if (metaTag.getAttribute("property") === property) {
                    metaTag.remove();
                    break;
                }
            }
        }
    }
    // dynamic-class-css
    attachCss(property, value, createClass, className) {
        let propertyShort = fastn_dom.propertyMap[property] || property;
        propertyShort = `__${propertyShort}`;
        let cls = `${propertyShort}-${fastn_dom.class_count}`;
        if (!!className) {
            cls = className;
        } else {
            if (!fastn_dom.unsanitised_classes[cls]) {
                fastn_dom.unsanitised_classes[cls] = ++fastn_dom.class_count;
            }
            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;
        }
        let cssClass = className ? cls : `.${cls}`;

        const obj = { property, value };

        if (value === undefined) {
            if (!ssr) {
                for (const className of this.#node.classList.values()) {
                    if (className.startsWith(`${propertyShort}-`)) {
                        this.#node.classList.remove(className);
                    }
                }
                this.#node.style[property] = null;
            }
            return cls;
        }

        if (!ssr && !doubleBuffering) {
            if (!!className) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] =
                        fastn_dom.classes[cssClass] || obj;
                    fastn_utils.createStyle(cssClass, obj);
                }
                return cls;
            }

            for (const className of this.#node.classList.values()) {
                if (className.startsWith(`${propertyShort}-`)) {
                    this.#node.classList.remove(className);
                }
            }

            if (createClass) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] =
                        fastn_dom.classes[cssClass] || obj;
                    fastn_utils.createStyle(cssClass, obj);
                }
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            } else if (!fastn_dom.classes[cssClass]) {
                if (typeof value === "object" && value !== null) {
                    for (let key in value) {
                        this.#node.style[key] = value[key];
                    }
                } else {
                    this.#node.style[property] = value;
                }
            } else {
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            }

            return cls;
        }

        fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;

        if (!!className) {
            return cls;
        }

        this.#node.classList.add(cls);
        return cls;
    }
    attachShadow(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("box-shadow", value);
            return;
        }

        const color = value.get("color");

        const lightColor = fastn_utils.getStaticValue(color.get("light"));
        const darkColor = fastn_utils.getStaticValue(color.get("dark"));

        const blur = fastn_utils.getStaticValue(value.get("blur"));
        const xOffset = fastn_utils.getStaticValue(value.get("x_offset"));
        const yOffset = fastn_utils.getStaticValue(value.get("y_offset"));
        const spread = fastn_utils.getStaticValue(value.get("spread"));
        const inset = fastn_utils.getStaticValue(value.get("inset"));

        const shadowCommonCss = `${
            inset ? "inset " : ""
        }${xOffset} ${yOffset} ${blur} ${spread}`;
        const lightShadowCss = `${shadowCommonCss} ${lightColor}`;
        const darkShadowCss = `${shadowCommonCss} ${darkColor}`;

        if (lightShadowCss === darkShadowCss) {
            this.attachCss("box-shadow", lightShadowCss, false);
        } else {
            let lightClass = this.attachCss("box-shadow", lightShadowCss, true);
            this.attachCss(
                "box-shadow",
                darkShadowCss,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    attachBackdropMultiFilter(value) {
        const filters = {
            blur: fastn_utils.getStaticValue(value.get("blur")),
            brightness: fastn_utils.getStaticValue(value.get("brightness")),
            contrast: fastn_utils.getStaticValue(value.get("contrast")),
            grayscale: fastn_utils.getStaticValue(value.get("grayscale")),
            invert: fastn_utils.getStaticValue(value.get("invert")),
            opacity: fastn_utils.getStaticValue(value.get("opacity")),
            sepia: fastn_utils.getStaticValue(value.get("sepia")),
            saturate: fastn_utils.getStaticValue(value.get("saturate")),
        };

        const filterString = Object.entries(filters)
            .filter(([_, value]) => !fastn_utils.isNull(value))
            .map(([name, value]) => `${name}(${value})`)
            .join(" ");

        this.attachCss("backdrop-filter", filterString, false);
    }
    attachTextShadow(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("text-shadow", value);
            return;
        }

        const color = value.get("color");

        const lightColor = fastn_utils.getStaticValue(color.get("light"));
        const darkColor = fastn_utils.getStaticValue(color.get("dark"));

        const blur = fastn_utils.getStaticValue(value.get("blur"));
        const xOffset = fastn_utils.getStaticValue(value.get("x_offset"));
        const yOffset = fastn_utils.getStaticValue(value.get("y_offset"));

        const shadowCommonCss = `${xOffset} ${yOffset} ${blur}`;
        const lightShadowCss = `${shadowCommonCss} ${lightColor}`;
        const darkShadowCss = `${shadowCommonCss} ${darkColor}`;

        if (lightShadowCss === darkShadowCss) {
            this.attachCss("text-shadow", lightShadowCss, false);
        } else {
            let lightClass = this.attachCss("box-shadow", lightShadowCss, true);
            this.attachCss(
                "text-shadow",
                darkShadowCss,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    getLinearGradientString(value) {
        var lightGradientString = "";
        var darkGradientString = "";

        let colorsList = value.get("colors").get().getList();
        colorsList.map(function (element) {
            // LinearGradient RecordInstance
            let lg_color = element.item;

            let color = lg_color.get("color").get();
            let lightColor = fastn_utils.getStaticValue(color.get("light"));
            let darkColor = fastn_utils.getStaticValue(color.get("dark"));

            lightGradientString = `${lightGradientString} ${lightColor}`;
            darkGradientString = `${darkGradientString} ${darkColor}`;

            let start = fastn_utils.getStaticValue(lg_color.get("start"));
            if (start !== undefined && start !== null) {
                lightGradientString = `${lightGradientString} ${start}`;
                darkGradientString = `${darkGradientString} ${start}`;
            }

            let end = fastn_utils.getStaticValue(lg_color.get("end"));
            if (end !== undefined && end !== null) {
                lightGradientString = `${lightGradientString} ${end}`;
                darkGradientString = `${darkGradientString} ${end}`;
            }

            let stop_position = fastn_utils.getStaticValue(
                lg_color.get("stop_position"),
            );
            if (stop_position !== undefined && stop_position !== null) {
                lightGradientString = `${lightGradientString}, ${stop_position}`;
                darkGradientString = `${darkGradientString}, ${stop_position}`;
            }

            lightGradientString = `${lightGradientString},`;
            darkGradientString = `${darkGradientString},`;
        });

        lightGradientString = lightGradientString.trim().slice(0, -1);
        darkGradientString = darkGradientString.trim().slice(0, -1);

        return [lightGradientString, darkGradientString];
    }
    attachLinearGradientCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("background-image", value);
            return;
        }

        const closure = fastn
            .closure(() => {
                let direction = fastn_utils.getStaticValue(
                    value.get("direction"),
                );

                const [lightGradientString, darkGradientString] =
                    this.getLinearGradientString(value);

                if (lightGradientString === darkGradientString) {
                    this.attachCss(
                        "background-image",
                        `linear-gradient(${direction}, ${lightGradientString})`,
                        false,
                    );
                } else {
                    let lightClass = this.attachCss(
                        "background-image",
                        `linear-gradient(${direction}, ${lightGradientString})`,
                        true,
                    );
                    this.attachCss(
                        "background-image",
                        `linear-gradient(${direction}, ${darkGradientString})`,
                        true,
                        `body.dark .${lightClass}`,
                    );
                }
            })
            .addNodeProperty(this, null, inherited);

        const colorsList = value.get("colors").get().getList();

        colorsList.forEach(({ item }) => {
            const color = item.get("color");

            [color.get("light"), color.get("dark")].forEach((variant) => {
                variant.addClosure(closure);
                this.#mutables.push(variant);
            });
        });
    }
    attachBackgroundImageCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("background-repeat", value);
            this.attachCss("background-position", value);
            this.attachCss("background-size", value);
            this.attachCss("background-image", value);
            return;
        }

        let src = fastn_utils.getStaticValue(value.get("src"));
        let lightValue = fastn_utils.getStaticValue(src.get("light"));
        let darkValue = fastn_utils.getStaticValue(src.get("dark"));

        let position = fastn_utils.getStaticValue(value.get("position"));
        let positionX = null;
        let positionY = null;
        if (position !== null && position instanceof Object) {
            positionX = fastn_utils.getStaticValue(position.get("x"));
            positionY = fastn_utils.getStaticValue(position.get("y"));

            if (positionX !== null) position = `${positionX}`;
            if (positionY !== null) {
                if (positionX === null) position = `0px ${positionY}`;
                else position = `${position} ${positionY}`;
            }
        }
        let repeat = fastn_utils.getStaticValue(value.get("repeat"));
        let size = fastn_utils.getStaticValue(value.get("size"));
        let sizeX = null;
        let sizeY = null;
        if (size !== null && size instanceof Object) {
            sizeX = fastn_utils.getStaticValue(size.get("x"));
            sizeY = fastn_utils.getStaticValue(size.get("y"));

            if (sizeX !== null) size = `${sizeX}`;
            if (sizeY !== null) {
                if (sizeX === null) size = `0px ${sizeY}`;
                else size = `${size} ${sizeY}`;
            }
        }

        if (repeat !== null) this.attachCss("background-repeat", repeat);
        if (position !== null) this.attachCss("background-position", position);
        if (size !== null) this.attachCss("background-size", size);

        if (lightValue === darkValue) {
            this.attachCss("background-image", `url(${lightValue})`, false);
        } else {
            let lightClass = this.attachCss(
                "background-image",
                `url(${lightValue})`,
                true,
            );
            this.attachCss(
                "background-image",
                `url(${darkValue})`,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    attachMaskImageCss(value, vendorPrefix) {
        const propertyWithPrefix = vendorPrefix
            ? `${vendorPrefix}-mask-image`
            : "mask-image";

        if (fastn_utils.isNull(value)) {
            this.attachCss(propertyWithPrefix, value);
            return;
        }

        let src = fastn_utils.getStaticValue(value.get("src"));
        let linearGradient = fastn_utils.getStaticValue(
            value.get("linear_gradient"),
        );
        let color = fastn_utils.getStaticValue(value.get("color"));

        const maskLightImageValues = [];
        const maskDarkImageValues = [];

        if (!fastn_utils.isNull(src)) {
            let lightValue = fastn_utils.getStaticValue(src.get("light"));
            let darkValue = fastn_utils.getStaticValue(src.get("dark"));

            const lightUrl = `url(${lightValue})`;
            const darkUrl = `url(${darkValue})`;

            if (!fastn_utils.isNull(linearGradient)) {
                const lightImageValues = [lightUrl];
                const darkImageValues = [darkUrl];

                if (!fastn_utils.isNull(color)) {
                    const lightColor = fastn_utils.getStaticValue(
                        color.get("light"),
                    );
                    const darkColor = fastn_utils.getStaticValue(
                        color.get("dark"),
                    );

                    lightImageValues.push(lightColor);
                    darkImageValues.push(darkColor);
                }
                maskLightImageValues.push(
                    `image(${lightImageValues.join(", ")})`,
                );
                maskDarkImageValues.push(
                    `image(${darkImageValues.join(", ")})`,
                );
            } else {
                maskLightImageValues.push(lightUrl);
                maskDarkImageValues.push(darkUrl);
            }
        }

        if (!fastn_utils.isNull(linearGradient)) {
            let direction = fastn_utils.getStaticValue(
                linearGradient.get("direction"),
            );

            const [lightGradientString, darkGradientString] =
                this.getLinearGradientString(linearGradient);

            maskLightImageValues.push(
                `linear-gradient(${direction}, ${lightGradientString})`,
            );
            maskDarkImageValues.push(
                `linear-gradient(${direction}, ${darkGradientString})`,
            );
        }

        const maskLightImageString = maskLightImageValues.join(", ");
        const maskDarkImageString = maskDarkImageValues.join(", ");

        if (maskLightImageString === maskDarkImageString) {
            this.attachCss(propertyWithPrefix, maskLightImageString, true);
        } else {
            let lightClass = this.attachCss(
                propertyWithPrefix,
                maskLightImageString,
                true,
            );
            this.attachCss(
                propertyWithPrefix,
                maskDarkImageString,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    attachMaskSizeCss(value, vendorPrefix) {
        const propertyNameWithPrefix = vendorPrefix
            ? `${vendorPrefix}-mask-size`
            : "mask-size";
        if (fastn_utils.isNull(value)) {
            this.attachCss(propertyNameWithPrefix, value);
        }
        const [size, ...two_values] = ["size", "size_x", "size_y"].map((size) =>
            fastn_utils.getStaticValue(value.get(size)),
        );

        if (!fastn_utils.isNull(size)) {
            this.attachCss(propertyNameWithPrefix, size, true);
        } else {
            const [size_x, size_y] = two_values.map((value) => value || "auto");
            this.attachCss(propertyNameWithPrefix, `${size_x} ${size_y}`, true);
        }
    }
    attachMaskMultiCss(value, vendorPrefix) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("mask-repeat", value);
            this.attachCss("mask-position", value);
            this.attachCss("mask-size", value);
            this.attachCss("mask-image", value);
            return;
        }

        const maskImage = fastn_utils.getStaticValue(value.get("image"));
        this.attachMaskImageCss(maskImage);
        this.attachMaskImageCss(maskImage, vendorPrefix);
        this.attachMaskSizeCss(value);
        this.attachMaskSizeCss(value, vendorPrefix);
        const maskRepeatValue = fastn_utils.getStaticValue(value.get("repeat"));
        if (fastn_utils.isNull(maskRepeatValue)) {
            this.attachCss("mask-repeat", maskRepeatValue, true);
            this.attachCss("-webkit-mask-repeat", maskRepeatValue, true);
        } else {
            this.attachCss("mask-repeat", maskRepeatValue, true);
            this.attachCss("-webkit-mask-repeat", maskRepeatValue, true);
        }
        const maskPositionValue = fastn_utils.getStaticValue(
            value.get("position"),
        );
        if (fastn_utils.isNull(maskPositionValue)) {
            this.attachCss("mask-position", maskPositionValue, true);
            this.attachCss("-webkit-mask-position", maskPositionValue, true);
        } else {
            this.attachCss("mask-position", maskPositionValue, true);
            this.attachCss("-webkit-mask-position", maskPositionValue, true);
        }
    }
    attachExternalCss(css) {
        if (!ssr) {
            let css_tag = document.createElement("link");
            css_tag.rel = "stylesheet";
            css_tag.type = "text/css";
            css_tag.href = css;

            let head =
                document.head || document.getElementsByTagName("head")[0];
            if (!fastn_dom.externalCss.has(css)) {
                head.appendChild(css_tag);
                fastn_dom.externalCss.add(css);
            }
        }
    }
    attachExternalJs(js) {
        if (!ssr) {
            let js_tag = document.createElement("script");
            js_tag.src = js;

            let head =
                document.head || document.getElementsByTagName("head")[0];
            if (!fastn_dom.externalJs.has(js)) {
                head.appendChild(js_tag);
                fastn_dom.externalCss.add(js);
            }
        }
    }
    attachColorCss(property, value, visited) {
        if (fastn_utils.isNull(value)) {
            this.attachCss(property, value);
            return;
        }
        value = value instanceof fastn.mutableClass ? value.get() : value;

        const lightValue = value.get("light");
        const darkValue = value.get("dark");

        const closure = fastn
            .closure(() => {
                let lightValueStatic = fastn_utils.getStaticValue(lightValue);
                let darkValueStatic = fastn_utils.getStaticValue(darkValue);

                if (lightValueStatic === darkValueStatic) {
                    this.attachCss(property, lightValueStatic, false);
                } else {
                    let lightClass = this.attachCss(
                        property,
                        lightValueStatic,
                        true,
                    );
                    this.attachCss(
                        property,
                        darkValueStatic,
                        true,
                        `body.dark .${lightClass}`,
                    );
                    if (visited) {
                        this.attachCss(
                            property,
                            lightValueStatic,
                            true,
                            `.${lightClass}:visited`,
                        );
                        this.attachCss(
                            property,
                            darkValueStatic,
                            true,
                            `body.dark  .${lightClass}:visited`,
                        );
                    }
                }
            })
            .addNodeProperty(this, null, inherited);

        [lightValue, darkValue].forEach((modeValue) => {
            modeValue.addClosure(closure);
            this.#mutables.push(modeValue);
        });
    }
    attachRoleCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("role", value);
            return;
        }
        value.addClosure(
            fastn
                .closure(() => {
                    let desktopValue = value.get("desktop");
                    let mobileValue = value.get("mobile");
                    if (
                        fastn_utils.sameResponsiveRole(
                            desktopValue,
                            mobileValue,
                        )
                    ) {
                        this.attachCss(
                            "role",
                            fastn_utils.getRoleValues(desktopValue),
                            true,
                        );
                    } else {
                        let desktopClass = this.attachCss(
                            "role",
                            fastn_utils.getRoleValues(desktopValue),
                            true,
                        );
                        this.attachCss(
                            "role",
                            fastn_utils.getRoleValues(mobileValue),
                            true,
                            `body.mobile .${desktopClass}`,
                        );
                    }
                })
                .addNodeProperty(this, null, inherited),
        );
        this.#mutables.push(value);
    }
    attachTextStyles(styles) {
        if (fastn_utils.isNull(styles)) {
            this.attachCss("font-style", styles);
            this.attachCss("font-weight", styles);
            this.attachCss("text-decoration", styles);
            return;
        }
        for (var s of styles) {
            switch (s) {
                case "italic":
                    this.attachCss("font-style", s);
                    break;
                case "underline":
                case "line-through":
                    this.attachCss("text-decoration", s);
                    break;
                default:
                    this.attachCss("font-weight", s);
            }
        }
    }
    attachAlignContent(value, node_kind) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("align-items", value);
            this.attachCss("justify-content", value);
            return;
        }
        if (node_kind === fastn_dom.ElementKind.Column) {
            switch (value) {
                case "top-left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "start");
                    break;
                case "top-center":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "center");
                    break;
                case "top-right":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "end");
                    break;
                case "left":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "start");
                    break;
                case "center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "center");
                    break;
                case "right":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "end");
                    break;
                case "bottom-left":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "left");
                    break;
                case "bottom-center":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "center");
                    break;
                case "bottom-right":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "end");
                    break;
            }
        }

        if (node_kind === fastn_dom.ElementKind.Row) {
            switch (value) {
                case "top-left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "start");
                    break;
                case "top-center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "start");
                    break;
                case "top-right":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "start");
                    break;
                case "left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "center");
                    break;
                case "center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "center");
                    break;
                case "right":
                    this.attachCss("justify-content", "right");
                    this.attachCss("align-items", "center");
                    break;
                case "bottom-left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "end");
                    break;
                case "bottom-center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "end");
                    break;
                case "bottom-right":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "end");
                    break;
            }
        }
    }

    attachImageSrcClosures(staticValue) {
        if (fastn_utils.isNull(staticValue)) return;

        if (staticValue instanceof fastn.recordInstanceClass) {
            let value = staticValue;
            let fields = value.getAllFields();

            let light_field_value = fastn_utils.flattenMutable(fields["light"]);
            light_field_value.addClosure(
                fastn
                    .closure(() => {
                        const is_dark_mode = ftd.dark_mode.get();
                        if (is_dark_mode) return;

                        const src =
                            fastn_utils.getStaticValue(light_field_value);
                        if (!ssr) {
                            let image_node = this.#node;
                            if (!fastn_utils.isNull(image_node)) {
                                if (image_node.nodeName.toLowerCase() === "a") {
                                    let childNodes = image_node.childNodes;
                                    childNodes.forEach(function (child) {
                                        if (
                                            child.nodeName.toLowerCase() ===
                                            "img"
                                        )
                                            image_node = child;
                                    });
                                }
                                image_node.setAttribute(
                                    "src",
                                    fastn_utils.getStaticValue(src),
                                );
                            }
                        } else {
                            this.attachAttribute(
                                "src",
                                fastn_utils.getStaticValue(src),
                            );
                        }
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(light_field_value);

            let dark_field_value = fastn_utils.flattenMutable(fields["dark"]);
            dark_field_value.addClosure(
                fastn
                    .closure(() => {
                        const is_dark_mode = ftd.dark_mode.get();
                        if (!is_dark_mode) return;

                        const src =
                            fastn_utils.getStaticValue(dark_field_value);
                        if (!ssr) {
                            let image_node = this.#node;
                            if (!fastn_utils.isNull(image_node)) {
                                if (image_node.nodeName.toLowerCase() === "a") {
                                    let childNodes = image_node.childNodes;
                                    childNodes.forEach(function (child) {
                                        if (
                                            child.nodeName.toLowerCase() ===
                                            "img"
                                        )
                                            image_node = child;
                                    });
                                }
                                image_node.setAttribute(
                                    "src",
                                    fastn_utils.getStaticValue(src),
                                );
                            }
                        } else {
                            this.attachAttribute(
                                "src",
                                fastn_utils.getStaticValue(src),
                            );
                        }
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(dark_field_value);
        }
    }

    attachLinkColor(value) {
        ftd.dark_mode.addClosure(
            fastn
                .closure(() => {
                    if (!ssr) {
                        const anchors =
                            this.#node.tagName.toLowerCase() === "a"
                                ? [this.#node]
                                : Array.from(this.#node.querySelectorAll("a"));
                        let propertyShort = `__${fastn_dom.propertyMap["link-color"]}`;

                        if (fastn_utils.isNull(value)) {
                            anchors.forEach((a) => {
                                a.classList.values().forEach((className) => {
                                    if (
                                        className.startsWith(
                                            `${propertyShort}-`,
                                        )
                                    ) {
                                        a.classList.remove(className);
                                    }
                                });
                            });
                        } else {
                            const lightValue = fastn_utils.getStaticValue(
                                value.get("light"),
                            );
                            const darkValue = fastn_utils.getStaticValue(
                                value.get("dark"),
                            );
                            let cls = `${propertyShort}-${JSON.stringify(
                                lightValue,
                            )}`;

                            if (!fastn_dom.unsanitised_classes[cls]) {
                                fastn_dom.unsanitised_classes[cls] =
                                    ++fastn_dom.class_count;
                            }

                            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;

                            const cssClass = `.${cls}`;

                            if (!fastn_dom.classes[cssClass]) {
                                const obj = {
                                    property: "color",
                                    value: lightValue,
                                };
                                fastn_dom.classes[cssClass] =
                                    fastn_dom.classes[cssClass] || obj;
                                let styles = document.getElementById("styles");
                                styles.innerHTML = `${
                                    styles.innerHTML
                                }${getClassAsString(cssClass, obj)}\n`;
                            }

                            if (lightValue !== darkValue) {
                                const obj = {
                                    property: "color",
                                    value: darkValue,
                                };
                                let darkCls = `body.dark ${cssClass}`;
                                if (!fastn_dom.classes[darkCls]) {
                                    fastn_dom.classes[darkCls] =
                                        fastn_dom.classes[darkCls] || obj;
                                    let styles =
                                        document.getElementById("styles");
                                    styles.innerHTML = `${
                                        styles.innerHTML
                                    }${getClassAsString(darkCls, obj)}\n`;
                                }
                            }

                            anchors.forEach((a) => a.classList.add(cls));
                        }
                    }
                })
                .addNodeProperty(this, null, inherited),
        );
        this.#mutables.push(ftd.dark_mode);
    }
    createClonedNode() {
        if (!doubleBuffering) {
            let node = this.#node;
            let clonedNode = node.cloneNode(true);
            this.#node = clonedNode;
            return node;
        }
    }

    replaceNodeWithClonedNode(node) {
        if (!doubleBuffering) {
            node.parentNode.replaceChild(this.#node, node);
        }
    }
    setStaticProperty(kind, value, inherited) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Children) {
            let originalNode = this.createClonedNode();
            if (fastn_utils.isWrapperNode(this.#tagName)) {
                let parentWithSibiling = this.#parent;
                if (Array.isArray(staticValue)) {
                    staticValue.forEach((func, index) => {
                        if (index !== 0) {
                            parentWithSibiling = new ParentNodeWithSibiling(
                                this.#parent.getParent(),
                                this.#children[index - 1],
                            );
                        }
                        this.#children.push(
                            fastn_utils.getStaticValue(func.item)(
                                parentWithSibiling,
                                inherited,
                            ),
                        );
                    });
                } else {
                    this.#children.push(
                        staticValue(parentWithSibiling, inherited),
                    );
                }
            } else {
                if (Array.isArray(staticValue)) {
                    staticValue.forEach((func) =>
                        this.#children.push(
                            fastn_utils.getStaticValue(func.item)(
                                this,
                                inherited,
                            ),
                        ),
                    );
                } else {
                    this.#children.push(staticValue(this, inherited));
                }
            }
            this.replaceNodeWithClonedNode(originalNode);
        } else if (kind === fastn_dom.PropertyKind.Id) {
            this.#node.id = staticValue;
        } else if (kind === fastn_dom.PropertyKind.BreakpointWidth) {
            if (fastn_utils.isNull(staticValue)) {
                return;
            }
            ftd.breakpoint_width.set(fastn_utils.getStaticValue(staticValue));
        } else if (kind === fastn_dom.PropertyKind.Css) {
            let css_list = staticValue.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            css_list.forEach((css) => {
                this.attachExternalCss(css);
            });
        } else if (kind === fastn_dom.PropertyKind.Js) {
            let js_list = staticValue.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            js_list.forEach((js) => {
                this.attachExternalJs(js);
            });
        } else if (kind === fastn_dom.PropertyKind.Width) {
            this.attachCss("width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Height) {
            fastn_utils.resetFullHeight();
            this.attachCss("height", staticValue);
            fastn_utils.setFullHeight();
        } else if (kind === fastn_dom.PropertyKind.Padding) {
            this.attachCss("padding", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingHorizontal) {
            this.attachCss("padding-left", staticValue);
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingVertical) {
            this.attachCss("padding-top", staticValue);
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingLeft) {
            this.attachCss("padding-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingRight) {
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingTop) {
            this.attachCss("padding-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingBottom) {
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Margin) {
            this.attachCss("margin", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginHorizontal) {
            this.attachCss("margin-left", staticValue);
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginVertical) {
            this.attachCss("margin-top", staticValue);
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginLeft) {
            this.attachCss("margin-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginRight) {
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginTop) {
            this.attachCss("margin-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginBottom) {
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderWidth) {
            this.attachCss("border-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopWidth) {
            this.attachCss("border-top-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomWidth) {
            this.attachCss("border-bottom-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftWidth) {
            this.attachCss("border-left-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightWidth) {
            this.attachCss("border-right-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRadius) {
            this.attachCss("border-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopLeftRadius) {
            this.attachCss("border-top-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopRightRadius) {
            this.attachCss("border-top-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomLeftRadius) {
            this.attachCss("border-bottom-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomRightRadius) {
            this.attachCss("border-bottom-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyle) {
            this.attachCss("border-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleVertical) {
            this.attachCss("border-top-style", staticValue);
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleHorizontal) {
            this.attachCss("border-left-style", staticValue);
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftStyle) {
            this.attachCss("border-left-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightStyle) {
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopStyle) {
            this.attachCss("border-top-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomStyle) {
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ZIndex) {
            this.attachCss("z-index", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Shadow) {
            this.attachShadow(staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextShadow) {
            this.attachTextShadow(staticValue);
        } else if (kind === fastn_dom.PropertyKind.BackdropFilter) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("backdrop-filter", staticValue);
                return;
            }

            let backdropType = staticValue[0];
            switch (backdropType) {
                case 1:
                    this.attachCss(
                        "backdrop-filter",
                        `blur(${fastn_utils.getStaticValue(staticValue[1])})`,
                    );
                    break;
                case 2:
                    this.attachCss(
                        "backdrop-filter",
                        `brightness(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 3:
                    this.attachCss(
                        "backdrop-filter",
                        `contrast(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 4:
                    this.attachCss(
                        "backdrop-filter",
                        `greyscale(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 5:
                    this.attachCss(
                        "backdrop-filter",
                        `invert(${fastn_utils.getStaticValue(staticValue[1])})`,
                    );
                    break;
                case 6:
                    this.attachCss(
                        "backdrop-filter",
                        `opacity(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 7:
                    this.attachCss(
                        "backdrop-filter",
                        `sepia(${fastn_utils.getStaticValue(staticValue[1])})`,
                    );
                    break;
                case 8:
                    this.attachCss(
                        "backdrop-filter",
                        `saturate(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 9:
                    this.attachBackdropMultiFilter(staticValue[1]);
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Mask) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("mask-image", staticValue);
                return;
            }

            const [backgroundType, value] = staticValue;

            switch (backgroundType) {
                case fastn_dom.Mask.Image()[0]:
                    this.attachMaskImageCss(value);
                    this.attachMaskImageCss(value, "-webkit");
                    break;
                case fastn_dom.Mask.Multi()[0]:
                    this.attachMaskMultiCss(value);
                    this.attachMaskMultiCss(value, "-webkit");
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Classes) {
            fastn_utils.removeNonFastnClasses(this);
            if (!fastn_utils.isNull(staticValue)) {
                let cls = staticValue.map((obj) =>
                    fastn_utils.getStaticValue(obj.item),
                );
                cls.forEach((c) => {
                    this.#node.classList.add(c);
                });
            }
        } else if (kind === fastn_dom.PropertyKind.Anchor) {
            // todo: this needs fixed for anchor.id = v
            // need to change position of element with id = v to relative
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("position", staticValue);
                return;
            }

            let anchorType = staticValue[0];
            switch (anchorType) {
                case 1:
                    this.attachCss("position", staticValue[1]);
                    break;
                case 2:
                    this.attachCss("position", staticValue[1]);
                    this.updateParentPosition("relative");
                    break;
                case 3:
                    const parent_node_id = staticValue[1];
                    this.attachCss("position", "absolute");
                    this.updatePositionForNodeById(parent_node_id, "relative");
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Sticky) {
            // sticky is boolean type
            switch (staticValue) {
                case "true":
                case true:
                    this.attachCss("position", "sticky");
                    break;
                case "false":
                case false:
                    this.attachCss("position", "static");
                    break;
                default:
                    this.attachCss("position", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Top) {
            this.attachCss("top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Bottom) {
            this.attachCss("bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Left) {
            this.attachCss("left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Right) {
            this.attachCss("right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Overflow) {
            this.attachCss("overflow", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowX) {
            this.attachCss("overflow-x", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowY) {
            this.attachCss("overflow-y", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Spacing) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("justify-content", staticValue);
                this.attachCss("gap", staticValue);
                return;
            }

            let spacingType = staticValue[0];
            switch (spacingType) {
                case fastn_dom.Spacing.SpaceEvenly[0]:
                case fastn_dom.Spacing.SpaceBetween[0]:
                case fastn_dom.Spacing.SpaceAround[0]:
                    this.attachCss("justify-content", staticValue[1]);
                    break;
                case fastn_dom.Spacing.Fixed()[0]:
                    this.attachCss(
                        "gap",
                        fastn_utils.getStaticValue(staticValue[1]),
                    );
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Wrap) {
            // sticky is boolean type
            switch (staticValue) {
                case "true":
                case true:
                    this.attachCss("flex-wrap", "wrap");
                    break;
                case "false":
                case false:
                    this.attachCss("flex-wrap", "no-wrap");
                    break;
                default:
                    this.attachCss("flex-wrap", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextTransform) {
            this.attachCss("text-transform", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextIndent) {
            this.attachCss("text-indent", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextAlign) {
            this.attachCss("text-align", staticValue);
        } else if (kind === fastn_dom.PropertyKind.LineClamp) {
            // -webkit-line-clamp: staticValue
            // display: -webkit-box, overflow: hidden
            // -webkit-box-orient: vertical
            this.attachCss("-webkit-line-clamp", staticValue);
            this.attachCss("display", "-webkit-box");
            this.attachCss("overflow", "hidden");
            this.attachCss("-webkit-box-orient", "vertical");
        } else if (kind === fastn_dom.PropertyKind.Opacity) {
            this.attachCss("opacity", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Cursor) {
            this.attachCss("cursor", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Resize) {
            // overflow: auto, resize: staticValue
            this.attachCss("resize", staticValue);
            this.attachCss("overflow", "auto");
        } else if (kind === fastn_dom.PropertyKind.Selectable) {
            if (staticValue === false) {
                this.attachCss("user-select", "none");
            } else {
                this.attachCss("user-select", null);
            }
        } else if (kind === fastn_dom.PropertyKind.MinHeight) {
            this.attachCss("min-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxHeight) {
            this.attachCss("max-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MinWidth) {
            this.attachCss("min-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxWidth) {
            this.attachCss("max-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.WhiteSpace) {
            this.attachCss("white-space", staticValue);
        } else if (kind === fastn_dom.PropertyKind.AlignSelf) {
            this.attachCss("align-self", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderColor) {
            this.attachColorCss("border-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftColor) {
            this.attachColorCss("border-left-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightColor) {
            this.attachColorCss("border-right-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopColor) {
            this.attachColorCss("border-top-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomColor) {
            this.attachColorCss("border-bottom-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.LinkColor) {
            this.attachLinkColor(staticValue);
        } else if (kind === fastn_dom.PropertyKind.Color) {
            this.attachColorCss("color", staticValue, true);
        } else if (kind === fastn_dom.PropertyKind.Background) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachColorCss("background-color", staticValue);
                this.attachBackgroundImageCss(staticValue);
                this.attachLinearGradientCss(staticValue);
                return;
            }

            let backgroundType = staticValue[0];
            switch (backgroundType) {
                case fastn_dom.BackgroundStyle.Solid()[0]:
                    this.attachColorCss("background-color", staticValue[1]);
                    break;
                case fastn_dom.BackgroundStyle.Image()[0]:
                    this.attachBackgroundImageCss(staticValue[1]);
                    break;
                case fastn_dom.BackgroundStyle.LinearGradient()[0]:
                    this.attachLinearGradientCss(staticValue[1]);
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Display) {
            this.attachCss("display", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Checked) {
            switch (staticValue) {
                case "true":
                case true:
                    this.attachAttribute("checked", "");
                    break;
                case "false":
                case false:
                    this.removeAttribute("checked");
                    break;
                default:
                    this.attachAttribute("checked", staticValue);
            }
            if (!ssr) this.#node.checked = staticValue;
        } else if (kind === fastn_dom.PropertyKind.Enabled) {
            switch (staticValue) {
                case "false":
                case false:
                    this.attachAttribute("disabled", "");
                    break;
                case "true":
                case true:
                    this.removeAttribute("disabled");
                    break;
                default:
                    this.attachAttribute("disabled", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextInputType) {
            this.attachAttribute("type", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextInputValue) {
            this.#rawInnerValue = staticValue;
            this.updateTextInputValue();
        } else if (kind === fastn_dom.PropertyKind.DefaultTextInputValue) {
            if (!fastn_utils.isNull(this.#rawInnerValue)) {
                return;
            }
            this.#rawInnerValue = staticValue;
            this.updateTextInputValue();
        } else if (kind === fastn_dom.PropertyKind.InputMaxLength) {
            this.attachAttribute("maxlength", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Placeholder) {
            this.attachAttribute("placeholder", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Multiline) {
            switch (staticValue) {
                case "true":
                case true:
                    this.updateTagName("textarea");
                    break;
                case "false":
                case false:
                    this.updateTagName("input");
                    break;
            }
            this.updateTextInputValue();
        } else if (kind === fastn_dom.PropertyKind.Download) {
            if (fastn_utils.isNull(staticValue)) {
                return;
            }
            this.attachAttribute("download", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Link) {
            // Changing node type to `a` for link
            // todo: needs fix for image links
            if (fastn_utils.isNull(staticValue)) {
                return;
            }
            this.updateToAnchor(staticValue);
        } else if (kind === fastn_dom.PropertyKind.LinkRel) {
            if (fastn_utils.isNull(staticValue)) {
                this.removeAttribute("rel");
            }
            let rel_list = staticValue.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            this.attachAttribute("rel", rel_list.join(" "));
        } else if (kind === fastn_dom.PropertyKind.OpenInNewTab) {
            // open_in_new_tab is boolean type
            switch (staticValue) {
                case "true":
                case true:
                    this.attachAttribute("target", "_blank");
                    break;
                default:
                    this.attachAttribute("target", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextStyle) {
            let styles = staticValue?.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            this.attachTextStyles(styles);
        } else if (kind === fastn_dom.PropertyKind.Region) {
            this.updateTagName(staticValue);
            if (this.#node.innerHTML) {
                this.#node.id = fastn_utils.slugify(this.#rawInnerValue);
            }
        } else if (kind === fastn_dom.PropertyKind.AlignContent) {
            let node_kind = this.#kind;
            this.attachAlignContent(staticValue, node_kind);
        } else if (kind === fastn_dom.PropertyKind.Loading) {
            this.attachAttribute("loading", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Src) {
            this.attachAttribute("src", staticValue);
        } else if (kind === fastn_dom.PropertyKind.SrcDoc) {
            this.attachAttribute("srcdoc", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ImageSrc) {
            this.attachImageSrcClosures(staticValue);
            ftd.dark_mode.addClosure(
                fastn
                    .closure(() => {
                        if (fastn_utils.isNull(staticValue)) {
                            this.attachAttribute("src", staticValue);
                            return;
                        }
                        const is_dark_mode = ftd.dark_mode.get();
                        const src = staticValue.get(
                            is_dark_mode ? "dark" : "light",
                        );
                        if (!ssr) {
                            let image_node = this.#node;
                            if (!fastn_utils.isNull(image_node)) {
                                if (image_node.nodeName.toLowerCase() === "a") {
                                    let childNodes = image_node.childNodes;
                                    childNodes.forEach(function (child) {
                                        if (
                                            child.nodeName.toLowerCase() ===
                                            "img"
                                        )
                                            image_node = child;
                                    });
                                }
                                image_node.setAttribute(
                                    "src",
                                    fastn_utils.getStaticValue(src),
                                );
                            }
                        } else {
                            this.attachAttribute(
                                "src",
                                fastn_utils.getStaticValue(src),
                            );
                        }
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Alt) {
            this.attachAttribute("alt", staticValue);
        } else if (kind === fastn_dom.PropertyKind.VideoSrc) {
            ftd.dark_mode.addClosure(
                fastn
                    .closure(() => {
                        if (fastn_utils.isNull(staticValue)) {
                            this.attachAttribute("src", staticValue);
                            return;
                        }
                        const is_dark_mode = ftd.dark_mode.get();
                        const src = staticValue.get(
                            is_dark_mode ? "dark" : "light",
                        );

                        this.attachAttribute(
                            "src",
                            fastn_utils.getStaticValue(src),
                        );
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Autoplay) {
            if (staticValue) {
                this.attachAttribute("autoplay", staticValue);
            } else {
                this.removeAttribute("autoplay");
            }
        } else if (kind === fastn_dom.PropertyKind.Muted) {
            if (staticValue) {
                this.attachAttribute("muted", staticValue);
            } else {
                this.removeAttribute("muted");
            }
        } else if (kind === fastn_dom.PropertyKind.Controls) {
            if (staticValue) {
                this.attachAttribute("controls", staticValue);
            } else {
                this.removeAttribute("controls");
            }
        } else if (kind === fastn_dom.PropertyKind.Loop) {
            if (staticValue) {
                this.attachAttribute("loop", staticValue);
            } else {
                this.removeAttribute("loop");
            }
        } else if (kind === fastn_dom.PropertyKind.Poster) {
            ftd.dark_mode.addClosure(
                fastn
                    .closure(() => {
                        if (fastn_utils.isNull(staticValue)) {
                            this.attachAttribute("poster", staticValue);
                            return;
                        }
                        const is_dark_mode = ftd.dark_mode.get();
                        const posterSrc = staticValue.get(
                            is_dark_mode ? "dark" : "light",
                        );

                        this.attachAttribute(
                            "poster",
                            fastn_utils.getStaticValue(posterSrc),
                        );
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Fit) {
            this.attachCss("object-fit", staticValue);
        } else if (kind === fastn_dom.PropertyKind.FetchPriority) {
            this.attachAttribute("fetchpriority", staticValue);
        } else if (kind === fastn_dom.PropertyKind.YoutubeSrc) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachAttribute("src", staticValue);
                return;
            }
            const id_pattern = "^([a-zA-Z0-9_-]{11})$";
            let id = staticValue.match(id_pattern);
            if (!fastn_utils.isNull(id)) {
                this.attachAttribute(
                    "src",
                    `https:\/\/youtube.com/embed/${id[0]}`,
                );
            } else {
                this.attachAttribute("src", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Role) {
            this.attachRoleCss(staticValue);
        } else if (kind === fastn_dom.PropertyKind.Code) {
            if (!fastn_utils.isNull(staticValue)) {
                let { modifiedText, highlightedLines } =
                    fastn_utils.findAndRemoveHighlighter(staticValue);
                if (highlightedLines.length !== 0) {
                    this.attachAttribute("data-line", highlightedLines);
                }
                staticValue = modifiedText;
            }
            let codeNode = this.#children[0].getNode();
            let codeText = fastn_utils.escapeHtmlInCode(staticValue);
            codeNode.innerHTML = codeText;
            this.#extraData.code = this.#extraData.code
                ? this.#extraData.code
                : {};
            fastn_utils.highlightCode(codeNode, this.#extraData.code);
        } else if (kind === fastn_dom.PropertyKind.CodeShowLineNumber) {
            if (staticValue) {
                this.#node.classList.add("line-numbers");
            } else {
                this.#node.classList.remove("line-numbers");
            }
        } else if (kind === fastn_dom.PropertyKind.CodeTheme) {
            this.#extraData.code = this.#extraData.code
                ? this.#extraData.code
                : {};
            if (fastn_utils.isNull(staticValue)) {
                if (!fastn_utils.isNull(this.#extraData.code.theme)) {
                    this.#node.classList.remove(this.#extraData.code.theme);
                }
                return;
            }
            if (!ssr) {
                fastn_utils.addCodeTheme(staticValue);
            }
            staticValue = fastn_utils.getStaticValue(staticValue);
            let theme = staticValue.replace(".", "-");
            if (this.#extraData.code.theme !== theme) {
                let codeNode = this.#children[0].getNode();
                this.#node.classList.remove(this.#extraData.code.theme);
                codeNode.classList.remove(this.#extraData.code.theme);
                this.#extraData.code.theme = theme;
                this.#node.classList.add(theme);
                codeNode.classList.add(theme);
                fastn_utils.highlightCode(codeNode, this.#extraData.code);
            }
        } else if (kind === fastn_dom.PropertyKind.CodeLanguage) {
            let language = `language-${staticValue}`;
            this.#extraData.code = this.#extraData.code
                ? this.#extraData.code
                : {};
            if (this.#extraData.code.language) {
                this.#node.classList.remove(language);
            }
            this.#extraData.code.language = language;
            this.#node.classList.add(language);
            let codeNode = this.#children[0].getNode();
            codeNode.classList.add(language);
            fastn_utils.highlightCode(codeNode, this.#extraData.code);
        } else if (kind === fastn_dom.PropertyKind.Favicon) {
            if (fastn_utils.isNull(staticValue)) return;
            this.setFavicon(staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaTitle
        ) {
            this.updateMetaTitle(staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGTitle
        ) {
            this.addMetaTagByProperty("og:title", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterTitle
        ) {
            this.addMetaTagByName("twitter:title", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaDescription
        ) {
            this.addMetaTagByName("description", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGDescription
        ) {
            this.addMetaTagByProperty("og:description", staticValue);
        } else if (
            kind ===
            fastn_dom.PropertyKind.DocumentProperties.MetaTwitterDescription
        ) {
            this.addMetaTagByName("twitter:description", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGImage
        ) {
            // staticValue is of ftd.raw-image-src RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByProperty("og:image");
                return;
            }
            this.addMetaTagByProperty(
                "og:image",
                fastn_utils.getStaticValue(staticValue.get("src")),
            );
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterImage
        ) {
            // staticValue is of ftd.raw-image-src RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("twitter:image");
                return;
            }
            this.addMetaTagByName(
                "twitter:image",
                fastn_utils.getStaticValue(staticValue.get("src")),
            );
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaThemeColor
        ) {
            // staticValue is of ftd.color RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("theme-color");
                return;
            }
            this.addMetaTagByName(
                "theme-color",
                fastn_utils.getStaticValue(staticValue.get("light")),
            );
        } else if (
            kind ===
            fastn_dom.PropertyKind.DocumentProperties
                .MetaFacebookDomainVerification
        ) {
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("facebook-domain-verification");
                return;
            }
            this.addMetaTagByName(
                "facebook-domain-verification",
                fastn_utils.getStaticValue(staticValue),
            );
        } else if (
            kind === fastn_dom.PropertyKind.IntegerValue ||
            kind === fastn_dom.PropertyKind.DecimalValue ||
            kind === fastn_dom.PropertyKind.BooleanValue
        ) {
            this.#node.innerHTML = staticValue;
            this.#rawInnerValue = staticValue;
        } else if (kind === fastn_dom.PropertyKind.StringValue) {
            this.#rawInnerValue = staticValue;
            staticValue = fastn_utils.markdown_inline(
                fastn_utils.escapeHtmlInMarkdown(staticValue),
            );
            staticValue = fastn_utils.process_post_markdown(
                this.#node,
                staticValue,
            );
            if (!fastn_utils.isNull(staticValue)) {
                this.#node.innerHTML = staticValue;
            } else {
                this.#node.innerHTML = "";
            }
        } else {
            throw "invalid fastn_dom.PropertyKind: " + kind;
        }
    }
    setProperty(kind, value, inherited) {
        if (value instanceof fastn.mutableClass) {
            this.setDynamicProperty(
                kind,
                [value],
                () => {
                    return value.get();
                },
                inherited,
            );
        } else if (value instanceof PropertyValueAsClosure) {
            this.setDynamicProperty(
                kind,
                value.deps,
                value.closureFunction,
                inherited,
            );
        } else {
            this.setStaticProperty(kind, value, inherited);
        }
    }
    setDynamicProperty(kind, deps, func, inherited) {
        let closure = fastn
            .closure(func)
            .addNodeProperty(this, kind, inherited);
        for (let dep in deps) {
            if (fastn_utils.isNull(deps[dep]) || !deps[dep].addClosure) {
                continue;
            }
            deps[dep].addClosure(closure);
            this.#mutables.push(deps[dep]);
        }
    }
    getNode() {
        return this.#node;
    }
    getExtraData() {
        return this.#extraData;
    }
    getChildren() {
        return this.#children;
    }
    mergeFnCalls(current, newFunc) {
        return () => {
            if (current instanceof Function) current();
            if (newFunc instanceof Function) newFunc();
        };
    }
    addEventHandler(event, func) {
        if (event === fastn_dom.Event.Click) {
            let onclickEvents = this.mergeFnCalls(this.#node.onclick, func);
            if (fastn_utils.isNull(this.#node.onclick))
                this.attachCss("cursor", "pointer");
            this.#node.onclick = onclickEvents;
        } else if (event === fastn_dom.Event.MouseEnter) {
            let mouseEnterEvents = this.mergeFnCalls(
                this.#node.onmouseenter,
                func,
            );
            this.#node.onmouseenter = mouseEnterEvents;
        } else if (event === fastn_dom.Event.MouseLeave) {
            let mouseLeaveEvents = this.mergeFnCalls(
                this.#node.onmouseleave,
                func,
            );
            this.#node.onmouseleave = mouseLeaveEvents;
        } else if (event === fastn_dom.Event.ClickOutside) {
            ftd.clickOutsideEvents.push([this, func]);
        } else if (!!event[0] && event[0] === fastn_dom.Event.GlobalKey()[0]) {
            ftd.globalKeyEvents.push([this, func, event[1]]);
        } else if (
            !!event[0] &&
            event[0] === fastn_dom.Event.GlobalKeySeq()[0]
        ) {
            ftd.globalKeySeqEvents.push([this, func, event[1]]);
        } else if (event === fastn_dom.Event.Input) {
            let onInputEvents = this.mergeFnCalls(this.#node.oninput, func);
            this.#node.oninput = onInputEvents;
        } else if (event === fastn_dom.Event.Change) {
            let onChangeEvents = this.mergeFnCalls(this.#node.onchange, func);
            this.#node.onchange = onChangeEvents;
        } else if (event === fastn_dom.Event.Blur) {
            let onBlurEvents = this.mergeFnCalls(this.#node.onblur, func);
            this.#node.onblur = onBlurEvents;
        } else if (event === fastn_dom.Event.Focus) {
            let onFocusEvents = this.mergeFnCalls(this.#node.onfocus, func);
            this.#node.onfocus = onFocusEvents;
        }
    }
    destroy() {
        for (let i = 0; i < this.#mutables.length; i++) {
            this.#mutables[i].unlinkNode(this);
        }
        // Todo: We don't need this condition as after destroying this node
        //  ConditionalDom reset this.#conditionUI to null or some different
        //  value. Not sure why this is still needed.
        if (!fastn_utils.isNull(this.#node)) {
            this.#node.remove();
        }
        this.#mutables = [];
        this.#parent = null;
        this.#node = null;
    }
}

class ConditionalDom {
    #marker;
    #parent;
    #node_constructor;
    #condition;
    #mutables;
    #conditionUI;

    constructor(parent, deps, condition, node_constructor) {
        this.#marker = fastn_dom.createKernel(
            parent,
            fastn_dom.ElementKind.Comment,
        );
        this.#parent = parent;

        this.#conditionUI = null;
        let closure = fastn.closure(() => {
            fastn_utils.resetFullHeight();
            if (condition()) {
                if (this.#conditionUI) {
                    let conditionUI = fastn_utils.flattenArray(
                        this.#conditionUI,
                    );
                    while (conditionUI.length > 0) {
                        let poppedElement = conditionUI.pop();
                        poppedElement.destroy();
                    }
                }
                this.#conditionUI = node_constructor(
                    new ParentNodeWithSibiling(this.#parent, this.#marker),
                );
                if (
                    !Array.isArray(this.#conditionUI) &&
                    fastn_utils.isWrapperNode(this.#conditionUI.getTagName())
                ) {
                    this.#conditionUI = this.#conditionUI.getChildren();
                }
            } else if (this.#conditionUI) {
                let conditionUI = fastn_utils.flattenArray(this.#conditionUI);
                while (conditionUI.length > 0) {
                    let poppedElement = conditionUI.pop();
                    poppedElement.destroy();
                }
                this.#conditionUI = null;
            }
            fastn_utils.setFullHeight();
        });
        deps.forEach((dep) => {
            if (!fastn_utils.isNull(dep) && dep.addClosure) {
                dep.addClosure(closure);
            }
        });

        this.#node_constructor = node_constructor;
        this.#condition = condition;
        this.#mutables = [];
    }

    getParent() {
        let nodes = [this.#marker];
        if (this.#conditionUI) {
            nodes.push(this.#conditionUI);
        }
        return nodes;
    }
}

fastn_dom.createKernel = function (parent, kind) {
    return new Node2(parent, kind);
};

fastn_dom.conditionalDom = function (
    parent,
    deps,
    condition,
    node_constructor,
) {
    return new ConditionalDom(parent, deps, condition, node_constructor);
};

class ParentNodeWithSibiling {
    #parent;
    #sibiling;
    constructor(parent, sibiling) {
        this.#parent = parent;
        this.#sibiling = sibiling;
    }
    getParent() {
        return this.#parent;
    }
    getSibiling() {
        return this.#sibiling;
    }
}

class ForLoop {
    #node_constructor;
    #list;
    #wrapper;
    #parent;
    #nodes;
    constructor(parent, node_constructor, list) {
        this.#wrapper = fastn_dom.createKernel(
            parent,
            fastn_dom.ElementKind.Comment,
        );
        this.#parent = parent;
        this.#node_constructor = node_constructor;
        this.#list = list;
        this.#nodes = [];

        fastn_utils.resetFullHeight();
        for (let idx in list.getList()) {
            this.createNode(idx, false);
        }
        fastn_utils.setFullHeight();
    }
    createNode(index, resizeBodyHeight = true) {
        if (resizeBodyHeight) {
            fastn_utils.resetFullHeight();
        }
        let parentWithSibiling = new ParentNodeWithSibiling(
            this.#parent,
            this.#wrapper,
        );
        if (index !== 0) {
            parentWithSibiling = new ParentNodeWithSibiling(
                this.#parent,
                this.#nodes[index - 1],
            );
        }
        let v = this.#list.get(index);
        let node = this.#node_constructor(parentWithSibiling, v.item, v.index);
        this.#nodes.splice(index, 0, node);
        if (resizeBodyHeight) {
            fastn_utils.setFullHeight();
        }
        return node;
    }
    createAllNode() {
        fastn_utils.resetFullHeight();
        this.deleteAllNode(false);
        for (let idx in this.#list.getList()) {
            this.createNode(idx, false);
        }
        fastn_utils.setFullHeight();
    }
    deleteAllNode(resizeBodyHeight = true) {
        if (resizeBodyHeight) {
            fastn_utils.resetFullHeight();
        }
        while (this.#nodes.length > 0) {
            this.#nodes.pop().destroy();
        }
        if (resizeBodyHeight) {
            fastn_utils.setFullHeight();
        }
    }
    getWrapper() {
        return this.#wrapper;
    }
    deleteNode(index) {
        fastn_utils.resetFullHeight();
        let node = this.#nodes.splice(index, 1)[0];
        node.destroy();
        fastn_utils.setFullHeight();
    }
    getParent() {
        return this.#parent;
    }
}

fastn_dom.forLoop = function (parent, node_constructor, list) {
    return new ForLoop(parent, node_constructor, list);
};
