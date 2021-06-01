import './FWCText.scss';

const LINE_SPACING = 23.5;
const LETTER_WIDTH = 13.19;

const FWCText = ({ text }) => {
    const lines = [];
    let spans = [];

    let color = 'white';
    let underlined = false;
    let flashing = false;
    let framed = false;

    const decorations = [];

    let buffer = '';
    let startCol = 0;
    let col = 0;
    for (let i = 0; i < text.length; i++) {
        const char = text[i];
        if (char === '\x1B' || char === '\r') {
            if (buffer !== '') {
                // close current part
                spans.push(
                    <tspan
                        key={buffer}
                        className={`fwc-color-${color}`}
                    >
                        {buffer}
                    </tspan>,
                );
                buffer = '';

                if (underlined) {
                    decorations.push(
                        <path
                            className={`fwc-underline fwc-color-${color}`}
                            d={`M ${startCol * LETTER_WIDTH + 1.5} ${lines.length * LINE_SPACING + 3.5} h ${(col - startCol) * LETTER_WIDTH - 1}`}
                            strokeLinecap="round"
                        />,
                    );
                }

                if (framed) {
                    decorations.push(
                        <path
                            className={`fwc-underline fwc-color-${color}`}
                            d={`M ${startCol * LETTER_WIDTH - 3} ${lines.length * LINE_SPACING - 17} h ${(col - startCol) * LETTER_WIDTH + 7} v 21 h ${-((col - startCol) * LETTER_WIDTH + 7)} v -21`}
                            strokeLinecap="round"
                        />,
                    );
                }

                startCol = col;
            }

            if (char === '\x1B') {
                let ctrlBuffer = '';
                i++;
                for (; i < text.length; i++) {
                    ctrlBuffer += text[i];

                    let match = true;
                    switch (ctrlBuffer) {
                    case 'm':
                        // Reset attribute
                        underlined = false;
                        flashing = false;
                        framed = false;
                        break;
                    case '4m':
                        // Underlined attribute
                        underlined = true;
                        break;
                    case ')m':
                        // Flashing attribute
                        flashing = true;
                        break;
                    case '\'m':
                        // Characters which follow must be framed
                        framed = true;
                        break;
                    case '<1m':
                        // Select YELLOW
                        color = 'yellow';
                        break;
                    case '<2m':
                        // Select RED
                        color = 'red';
                        break;
                    case '<3m':
                        // Select GREEN
                        color = 'green';
                        break;
                    case '<4m':
                        // Select AMBER
                        color = 'amber';
                        break;
                    case '<5m':
                        // Select CYAN (blue-green)
                        color = 'cyan';
                        break;
                    case '<6m':
                        // Select MAGENTA
                        color = 'magenta';
                        break;
                    case '<7m':
                        // Select WHITE
                        color = 'white';
                        break;
                    default:
                        match = false;
                        break;
                    }

                    if (match) {
                        break;
                    }
                }

                continue;
            }

            if (char === '\r') {
                lines.push(<text className="fwc-text" y={lines.length * LINE_SPACING}>{spans}</text>);

                spans = [];
                col = 0;
                startCol = 0;
                continue;
            }
        }

        buffer += char;
        col++;
    }

    if (buffer !== '') {
        spans.push(
            <tspan
                key={buffer}
                className={`fwc-color-${color}`}
            >
                {buffer}
            </tspan>,
        );
    }

    if (spans.length) {
        lines.push(<text className="fwc-text" y={lines.length * LINE_SPACING}>{spans}</text>);
    }

    return (
        <>
            {lines}
            {decorations}
        </>
    );
};

export default FWCText;
