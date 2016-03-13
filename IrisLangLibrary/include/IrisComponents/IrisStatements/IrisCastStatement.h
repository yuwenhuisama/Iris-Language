#ifndef _H_IRISCASTSTATEMENT_
#define _H_IRISCASTSTATEMENT_
#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisExpression;
class IrisCastStatement :
	public IrisStatement
{
protected:
	IrisList<IrisExpression*>* m_pParameters = nullptr;

public:

	virtual bool Generate();

	IrisCastStatement(IrisList<IrisExpression*>* pParameters);
	virtual ~IrisCastStatement();
};

#endif