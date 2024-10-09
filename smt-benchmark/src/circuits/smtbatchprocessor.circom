pragma circom 2.1.6;

include "../../node_modules/circomlib/circuits/smt/smtprocessor.circom";

template SMTBatchProcessor (nLevels, nOperations) {
    signal input oldRoot;
    signal output newRoot;
    signal input siblings[nOperations][nLevels]; // Each operation has its own siblings array
    signal input oldKey[nOperations]; // Each operation gets its own key
    signal input oldValue[nOperations]; // Each operation gets its own value
    signal input isOld0[nOperations]; // Each operation has its own flag
    signal input newKey[nOperations]; // Each operation gets its own newKey
    signal input newValue[nOperations]; // Each operation gets its own newValue
    signal input fnc[nOperations][2]; // Each operation gets its own function signal

    // Array to hold SMTProcessor components
    component smtProcessors[nOperations];

    // Loop to initialize and connect each SMTProcessor
    for (var i = 0; i < nOperations; i++) {
        smtProcessors[i] = SMTProcessor(nLevels);
        
        // Assign input signals for each SMTProcessor instance
        if (i == 0) {
            smtProcessors[i].oldRoot <== oldRoot;       // Use the oldRoot signal
        } else {
            smtProcessors[i].oldRoot <== smtProcessors[i - 1].newRoot;       // Use the previous newRoot
        }
        smtProcessors[i].siblings <== siblings[i];     // Use the i-th siblings array
        smtProcessors[i].oldKey <== oldKey[i];         // Use the i-th oldKey
        smtProcessors[i].oldValue <== oldValue[i];     // Use the i-th oldValue
        smtProcessors[i].isOld0 <== isOld0[i];         // Use the i-th isOld0 flag
        smtProcessors[i].newKey <== newKey[i];         // Use the i-th newKey
        smtProcessors[i].newValue <== newValue[i];     // Use the i-th newValue
        smtProcessors[i].fnc <== fnc[i];               // Use the i-th fnc signal
    }

    // Assign output signals
    newRoot <== smtProcessors[nOperations - 1].newRoot; // Use the last newRoot
}