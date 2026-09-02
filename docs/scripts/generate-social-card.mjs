import path from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';

const docsRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(docsRoot, 'src/assets/draupnir-social-card.svg');
const output = path.join(docsRoot, 'public/draupnir-social-card.png');

await sharp(source).png({ compressionLevel: 9, palette: true }).toFile(output);
