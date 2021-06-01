import * as React from 'react';
import './WarningDisplay.scss';
import { useSimVar } from "../../util";
import FWCText from "../FWCText/FWCText.jsx";
import ReactDOM from "react-dom";

console.log("hello!");

const WarningDisplay = () => {
    const flightPhase = +useSimVar("L:A32NX_FWC_FLIGHT_PHASE", "number");

    const message = [
        "",
        "\x1b<4m\x1b4mFWS\x1bm FWC 1+2 FAULT",
        "\x1b<5m -MONITOR SYS",
        "\x1b<5m -MONITOR OVERHEAD PANEL",
    ].join("\r");

    /*const message = [
        "\x1b<2m\x1b4mAUTO FLT\x1bm AP OFF",
        "\x1b<2m\x1b4mELEC\x1bm \x1b'mEMER CONFIG\x1bm",
        "\x1b<5m PROC:GRVTY FUEL FEEDING",
        "\x1b<5m -FAC 1......OFF THEN ON",
        "\x1b<5m -GEN 1 + 2....OFF THEN ON",
        "\x1b<5m -BUS TIE............OFF",
        "\x1b<5m -GEN 1+2... OFF THEN ON",
    ].join("\r");

    const message = [
        "\x1b<3m\x1b4mT.O\x1bm AUTO BRK\x1b<5m.....MAX",
        "\x1b<3m    SIGNS\x1b<5m.........ON",
        "\x1b<3m    CABIN\x1b<5m......CHECK",
        "\x1b<3m    SPLRS\x1b<5m........ARM",
        "\x1b<3m    FLAPS\x1b<5m........T.O",
        "\x1b<3m    T.O CONFIG\x1b<5m..TEST",
    ].join("\r");*/

    /*const message = [
        "023456789012345678901234567",
        " -2",
        " -3",
        " -4",
        " -5",
        " -6",
        " -7",
        " -8",
    ].join("\r");*/

    const showAdvisory = false;
    const showOverflow = true;
    const showStatus = false;

    return (
        <g>
            <path className="ewd-ecam-line--outer" d="M 3   406 h 342" strokeLinecap="round" />
            <path className="ewd-ecam-line" d="M 3   406 h 342" strokeLinecap="round" />

            <path className="ewd-ecam-line--outer" d="M 409 406 h 188" strokeLinecap="round" />
            <path className="ewd-ecam-line" d="M 409 406 h 188" strokeLinecap="round" />

            <path className="ewd-ecam-line--outer" d="M 378.5 422 v 146" strokeLinecap="round" />
            <path className="ewd-ecam-line" d="M 378.5 422 v 146" strokeLinecap="round" />

            <g className="ewd-warning-text-left" transform="translate(7 432)" fill="white">
                <FWCText text={message} />
            </g>

            <g className="ewd-warning-text-left" transform="translate(407 432)" fill="white">
                {/*<FWCText
                    text={[
                        '\x1b<2mLAND ASAP',
                        '\x1b<4mENG 1',
                        '\x1b<4mENG 2',
                        '\x1b<4mF/CTL',
                        '\x1b<4mAUTO FLT',
                    ].join('\r')}
                />*/}
                <FWCText
                    text={[
                        '\x1b4m NOT AVAIL',
                        '\x1b<2mECAM WARN',
                        '\x1b<2mALTI ALERT',
                        '\x1b<2mSTATUS',
                        '\x1b<2mA/CALL OUT',
                        '\x1b<2mMEMO',
                    ].join('\r')}
                />
            </g>

            {showAdvisory ? (
                <g>
                    <text
                        x="378"
                        y="414"
                        fill="white"
                        textAnchor="middle"
                        style={{ fontSize: '1.30em', letterSpacing: '0.07em' }}
                    >
                        ADV
                    </text>
                    <path
                        className="ewd-underline color-white"
                        d="M 358 418 h 40 v -20 h -40 v 20"
                        strokeLinecap="round"
                    />
                </g>
            ) : null}

            {showOverflow ? (
                <path
                    d="m 376 571 h 5 v 15 h 5 l -7.5,11 l -7.5,-11 h 5 v -15"
                    style={{
                        fill: '#00ff00',
                        stroke: 'none',
                    }}
                />
            ) : null}

            {showStatus ? (
                <g>
                    <text x="378.5" y="587.5" fill="white" textAnchor="middle" style={{ fontSize: '1.25em' }}>
                        STS
                    </text>
                    <path
                        className="ewd-underline color-white"
                        d="M 361.5 590.5 h 34 v -18 h -34 v 18"
                        strokeLinecap="round"
                    />
                </g>
            ) : null}
        </g>
    );
};

const renderTarget = document.getElementById('A32NX_WIDGET_WD_REACT_MOUNT');

console.log("hello!");

ReactDOM.render(<WarningDisplay />, renderTarget);
