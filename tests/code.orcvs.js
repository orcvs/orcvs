bpm(110);

const lerper = lerp(1, 10);

on.ptn('▮▯▯▯▯▯▯', on => {
  const cycler = cycle(1, 10);
  console.log({ cycler: cycler() });
  on.ptn('▮▯', on => {
    on.ptn('▮▯▮▯', on => { });
  });
})

// const lerper = lerp(1, 10);

// ptn('▮▯▯▯▯▯▯', () => {
//   const cycler = cycle(10);
//   console.log({cycler: cycler()});
//   console.log({lerper: lerper()});
//   play(10, D2({d: 4}));
// })

// ptn('▮▯▯▯▯▯▯', () => {
//   play(10, C1({d: 4}));
// });
