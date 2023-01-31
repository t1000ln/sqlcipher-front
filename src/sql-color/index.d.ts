interface HighlightOptions {
    html?: boolean;
    htmlEscaper?: (str: string) => string
    classPrefix?: string;
    colors?: {
        keyword: string;
        function: string;
        number: string;
        string: string;
        special: string;
        bracket: string;
        clear: string;
    };
}

interface Segment {
    name: string;
    content: string;
}

function getSegments(sqlString: string): Array<Segment>;

function highlight(sqlString: string, options?: HighlightOptions): string;

export default highlight;
export {getSegments, HighlightOptions, Segment};
