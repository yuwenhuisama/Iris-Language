#include "IrisInterpreter/IrisNativeClasses/IrisClosureBlockBaseTag.h"



IrisClosureBlockBaseTag::IrisClosureBlockBaseTag(IIrisClosureBlock* pClosureBlock) : m_pClosureBlock(pClosureBlock)
{
}

IIrisClosureBlock* IrisClosureBlockBaseTag::GetClosureBlock() {
	return m_pClosureBlock;
}

void IrisClosureBlockBaseTag::SetClosureBlock(IIrisClosureBlock* pClosureBlock) {
	m_pClosureBlock = pClosureBlock;
}

IrisClosureBlockBaseTag::~IrisClosureBlockBaseTag()
{
}
