import './Sts.scss';
import ReactDOM from 'react-dom';
import React, { useEffect, useState } from 'react';
import { getRenderTarget, setIsEcamPage } from '../../../Common/defaults';
import { SimVarProvider, useSimVar } from '../../../Common/simVars';
import FWCText from "../../../EWD/FWCText/FWCText";

setIsEcamPage('sts_page');

export const StsPage = () => {
    return (
        <svg id="ecam-sts" viewBox="0 0 600 600" style={{ marginTop: '-60px' }} xmlns="http://www.w3.org/2000/svg">
            <text
                id="pageTitle"
                className="PageTitle"
                x="300"
                y="18"
                textAnchor="middle"
                alignmentBaseline="central"
            >
                STATUS
            </text>

            <path className="sts-line" d="M 378.5 70 v 400" />

            {/*<g className="status-text-left" transform="translate(7 84)" fill="white">
                <FWCText
                    text={[
                        '\x1b<5mMIN RAT SPEED.....140 KT',
                        '',
                        '\x1b<3mENG 1 N1 DEGRADED MODE',
                        '\x1b<3mENG 2 N1 DEGRADED MODE',
                        '\x1b<3mBOTH PFD ON SAME FAC',
                        '\x1b<3mCAT 2 ONLY',
                    ].join('\r')}
                />
            </g>

            <g className="status-text-right" transform="translate(407 84)" fill="white">
                <FWCText
                    text={[
                        '   \x1b4mINOP SYS\x1bm',
                        '\x1b<4mA/THR',
                        '\x1b<4mCAT 3',
                        '\x1b<4mFAC 2',
                        '\x1b<4mVENT BLOWER',
                        '\x1b<4mMAIN GALLEY',
                        '\x1b<4mGEN 1',
                        '\x1b<4mGEN 2',
                        '\x1b<4mGPWS',
                        '\x1b<4mGPWS TERR',
                        '\x1b<4mROW/ROP',
                    ].join('\r')}
                />
            </g>*/}

            <g className="status-text-left" transform="translate(7 84)" fill="white">
                <FWCText
                    text={[
                        '\x1b<5mMAX SPEED.........320 KT',
                        '',
                        '\x1b<7m\x1b4mAPPR PROC:\x1bm',
                        ' \x1b<5m-FOR LDG.....USE FLAP 3',
                        ' \x1b<5m-GPWS LDG FLAP 3.....ON',
                        '',
                        '\x1b<5mLDG DIST PROC......APPLY',
                        '',
                        '\x1b<3mALTN LAW : PROT LOST',
                        '\x1b<3mWHEN L/G DN : DIRECT LAW',
                    ].join('\r')}
                />
            </g>

            <g className="status-text-right" transform="translate(407 84)" fill="white">
                <FWCText
                    text={[
                        '   \x1b4mINOP SYS\x1bm',
                        '\x1b<4mF/CTL PROT',
                        '\x1b<4mAP 1+2',
                        '\x1b<4mA/THR',
                        '\x1b<4mCAT 2',
                        '\x1b<4mYAW DAMPER 2',
                    ].join('\r')}
                />
            </g>
        </svg>
    );
};

ReactDOM.render(<SimVarProvider><StsPage /></SimVarProvider>, getRenderTarget());
