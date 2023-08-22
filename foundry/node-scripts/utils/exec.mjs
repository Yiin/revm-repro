import { exec as bashExec } from 'child_process';

export function exec(command) {
    return new Promise((resolve, reject) => {
        bashExec(command, (error, stdout, stderr) => {
          if (error) {
            reject(error);
            return;
          }
          resolve(stdout.toString());
        });
      });
}
