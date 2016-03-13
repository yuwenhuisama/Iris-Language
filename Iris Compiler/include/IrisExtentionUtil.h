#ifndef _H_IRISEXTENTIONUTIL_
#define _H_IRISEXTENTIONUTIL_

typedef struct IR_ExtentionInitializeStruct_tag {
	void* m_pInterpreter;
	void* m_pCompiler;
	void* m_pFatalErrorHandler;
	void* m_pGC;
	void* m_pThreadManager;
	void* m_pMemoryPool;
	void* m_pMemoryPoolList;
} IR_ExtentionInitializeStruct, *PIR_ExtentionInitializeStruct;

#endif