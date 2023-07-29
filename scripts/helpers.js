import { spawn } from 'node:child_process';
import { blue, bold, cyan, green, magenta, red, yellow } from './colors.js';

const colors = [cyan, yellow, blue, red, green, magenta];
let nextColorIndex = 0;

export function runWithEcho(command, parameters, options) {
	const color = colors[nextColorIndex];
	nextColorIndex = (nextColorIndex + 1) % colors.length;
	return new Promise((resolve, reject) => {
		const cmdString = formatCommand(command, parameters);
		console.error(bold(`\n${color`Run>`} ${cmdString}`));

		const childProcess = spawn(command, parameters, { stdio: 'inherit', ...options });

		childProcess.on('close', code => {
			if (code) {
				reject(new Error(`"${cmdString}" exited with code ${code}.`));
			} else {
				console.error(bold(`${color`Finished>`} ${cmdString}\n`));
				resolve();
			}
		});
	});
}

export function runAndGetStdout(command, parameters) {
	return new Promise((resolve, reject) => {
		const childProcess = spawn(command, parameters);
		let stdout = '';

		childProcess.stderr.pipe(process.stderr);
		childProcess.stdout.on('data', data => (stdout += String(data)));

		childProcess.on('close', code => {
			if (code) {
				reject(new Error(`"${formatCommand(command, parameters)}" exited with code ${code}.`));
			} else {
				resolve(stdout.trim());
			}
		});
	});
}

function formatCommand(command, parameters) {
	return [command, ...parameters].join(' ');
}
