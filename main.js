let count = 1;

const interval = setInterval(() => {
  // clear the last line
  process.stdout.clearLine();
  process.stdout.cursorTo(0);

  // print the current number
  console.log(count);

  // if we've reached 100, stop the interval
  if (count === 100) {
    console.log(`${count}/100`);
    clearInterval(interval);
    return;
  }

  // print the status bar
  process.stdout.write(`${count}/100`);

  count++;
}, 1000); // 1000 milliseconds = 1 second
