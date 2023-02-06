import { Orcvs } from './orcvs'




async function main(): Promise<void> {
    const orcvs = Orcvs();

    console.info('Welcome to Orcvs');

    await orcvs.init();
    await orcvs.start();


    console.log("HERE");
}
  
if (require.main === module) {
    main()
        .then( () => console.info("Terminated") )
        .catch( (err) => console.error(err) );
}

