#ifndef _H_IRISEXTENTIONUTIL_
#define _H_IRISEXTENTIONUTIL_

class IrisInterpreter;
class IrisCompiler;
class IrisGC;
class IrisThreadManager;
class IrisFatalErrorHandler;

typedef struct IR_ExtentionInitializeStruct_tag {
	IrisInterpreter* m_pInterpreter;
	IrisCompiler* m_pCompiler;
	IrisFatalErrorHandler* m_pFatalErrorHandler;
	IrisGC* m_pGC;
	IrisThreadManager* m_pThreadManager;
	//void* m_pMemoryPool;
	//void* m_pMemoryPoolList;
} IR_ExtentionInitializeStruct, *PIR_ExtentionInitializeStruct;

#endif