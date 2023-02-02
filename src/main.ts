
import vm from 'node:vm';


const code = `
    console.log('hello');
`;


async function main(): Promise<void> {
    console.log("main");

    // const orcvs = new Orcvs();
    // await orcvs.init();


    // const context = { midi: orcvs.midi };
    // vm.createContext(context); 

    // // clock.tick( () => {  
    //     vm.runInContext(code, context);
    // // });

    // await new Promise((r) => setTimeout(r, 1000));
    // await orcvs.stop();
    // Promise.resolve();
}
  
if (require.main === module) {
    main()
        .then( () => console.info("End") )
        .catch( (err) => console.error(err) );
}

