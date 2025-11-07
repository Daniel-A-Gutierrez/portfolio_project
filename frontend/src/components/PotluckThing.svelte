<div class="pico potluck-planner center">
    {#if componentState == State.FindOrCreate}
        <h1><center>Potluck Planner</center></h1>
        <div> 
            <!-- svelte-ignore a11y-no-redundant-roles -->
            <fieldset role="group">
                <input type="text" placeholder="Party Code" bind:value={searchUUID}/>
                <button onclick={() => findParty(searchUUID)}>Find</button>
            </fieldset>
            <center class="even"> or </center>
            <center><button onclick={createNew}>Create New</button></center>
        </div>
    {:else if componentState == State.Error}
        <button onclick={reset}>Back</button>
        <p class="error">Error: {error}</p>
    {:else if componentState == State.CreateNew}
        <button onclick={reset}>Back</button>
        <div>
            <h1><center>Create A Party</center></h1>
            <input type="text" placeholder="Party Name" bind:value={partyName}/>
            <input type="date" bind:value={date}/>
            <input type="time" bind:value={time} placeholder="time"/>
            <input type="text" placeholder="Location" bind:value={location}/>
            <center><button onclick = {createParty}>Create</button></center>
        </div>
    {:else if componentState == State.PartyCreated}
        <center>    
            <h1>Success!</h1>
            <h2>Your party's code is below, save it!</h2>
            <h2>{partyUUID}</h2>
            <button onclick={()=>findParty(partyUUID)}>View</button>
        </center>
    {:else if componentState == State.Party}
        <button onclick={reset}>Back</button>
        <center>
            <h1>{partyName}</h1>
            <table>
                <thead>
                    <tr>
                        <td>Participant</td>
                        <td>Food Item</td>
                    </tr>   
                </thead>
                <tbody>
                    {#each items as item}
                        <tr>
                            <td>{item.name}</td>
                            <td>{item.item}</td>
                        </tr>
                    {/each}
                    
                </tbody>
            </table>

            <input type="text" placeholder="Your name" bind:value={yourname}/>
            <input type="text" placeholder="What you're bringing (you can't edit this later!)" bind:value={whatyourebringing}/>
            <button onclick={commit}>Commit</button>
            
            <center><small>{partyUUID}</small></center>

        </center>
    {/if}
</div>

<script lang="ts">
    import {KVGet,KVSet,KVAppend} from '../lib.ts';
 
    enum State 
    {
        FindOrCreate,
        Error,
        CreateNew,
        PartyCreated,
        Party
    }

    type PotluckEntry  =
    {
        uuid : string,
        name : string,
        date : Date,
        time : any,
        location : string,
    }

    //a potluck's items always has the key "{potluck.uuid}.items"
    type ItemsEntry = 
    {
        name : string,
        item : string
    };


    let componentState : State = $state(State.CreateNew);
    let searchUUID = $state("");
    let error = $state("bad things!");
    let partyName = $state("Fake Party");
    let date : Date = $state(new Date());
    let time : any = $state("");
    let location = $state("");
    let partyUUID = $state("");
    let items : {name:string,item:string}[]= $state([{name: "lizbeth",item : "nachos"}]);
    let yourname = $state("");
    let whatyourebringing = $state("");

    function reset() 
    {
        searchUUID = "";
        componentState = State.FindOrCreate;
    }

    async function findParty(uuid : string)
    {
        let res = KVGet<PotluckEntry>(uuid);
        let items_res = KVGet<ItemsEntry[]>(uuid + ".items");
        let party,party_items;
        try 
        {
            [party,party_items] = await Promise.all([res,items_res]);
        }
        catch (e) 
        {
            console.error(e);
            error = "Failed to search for potluck.";
            componentState = State.Error;
            return;
        }
        if (party == null || party_items == null)
        {
            componentState = State.Error;
            error = "Party not found";
            return;
        }
        partyName = party.name;
        partyUUID = party.uuid;
        location = party.location;
        date = party.date;
        time = party.time;
        items = party_items;
        componentState = State.Party;
    }

    async function createNew()
    {
        componentState = State.CreateNew;
    }

    async function createParty()
    {
        partyUUID = crypto.randomUUID();
        console.log(partyUUID);
        let new_party : PotluckEntry = 
        {
            uuid : partyUUID,
            name : partyName,
            date : date,
            time : time,
            location : location
        };
        try 
        {
            await Promise.all([KVSet(partyUUID, new_party), KVSet(partyUUID + ".items", [])]);
        }
        catch (e) 
        {
            console.error(e); 
            error = "Failed to create new potluck"; 
            componentState = State.Error;
            return;
        }
        componentState = State.PartyCreated;
    }

    async function commit()
    {
        let entry : ItemsEntry = {name: yourname, item : whatyourebringing};
        try {await KVAppend(partyUUID + '.items', entry );}
        catch (e)
        {
            console.error(e);
            error = "Failed to append your entry to the items list.";
            componentState = State.Error;
        }   
        items = [...items, entry];
        yourname = "";
        whatyourebringing = "";
    }

</script>

<style>
    .potluck-planner
    {
        max-width: 576px;
        margin: 0 auto 0 auto;
        padding:1rem;
        box-sizing: border-box;
        border-radius: 1rem;
        border: 1.5px solid gray;
    }

    .even{margin-bottom:16px;}
    .error{color: red;}
</style>