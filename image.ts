import * as fs from 'fs';
import * as path from 'path';

import {buildCompositedImage, BuildCompositedImageOptions} from './index';

const productImagePath: string = path.join(__dirname, './resources/product.jpg');
const productImageBuffer: Buffer = fs.readFileSync(productImagePath);

const overlayImagePath: string = path.join(__dirname, './resources/overlay.png');
const overlayImageBuffer: Buffer = fs.readFileSync(overlayImagePath);

const options: BuildCompositedImageOptions = {
    backgroundColor: [0, 0, 255, 255],
    resizeMode: {
        type: 'Scale',
        value: 1
    },
    offsetMode: {
        type: 'Percent',
        value: [50, 50]
    }
};

buildCompositedImage(productImageBuffer, overlayImageBuffer, options);
