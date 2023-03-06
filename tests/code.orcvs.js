bpm(110);

const lerper = lerp(1, 10);

ptn('▮▯▯▯▯▯▯▯', () => {
  // const cycler = cycle(1, 10);
  // console.log({ cycler: cycler() });
  // play(10, D2({d: 4}));
  play(1, C({d: 15, a: z, r: 1}));
  ptn('▮▯▯▯', () => {
    ptn('▮▯▮▯', () => { });
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
